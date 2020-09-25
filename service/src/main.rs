/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dbus_crate::{
    blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged, message::SignalArgs,
};
use log::{info, trace, warn};
use snafu::{ResultExt, Snafu};

use std::io::{Read, Seek, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Mutex;

use std::time::Duration;

mod cleaner;
mod config;
mod constants;
mod dbus;
mod ec_control;
mod nbfc;
mod state;
mod temp;

use cleaner::cleaner;
use config::{nbfc_control::load_control_config, service::ServiceConfig};
use constants::{BUS_NAME, DBUS_PATH};
use dbus::connection::create_dbus_conn;
use ec_control::ec_manager::{ECError, ECManager};
use state::State;

const CRITICAL_INTERVAL: u8 = 10;

// TODO: The error string is not displayed at the end of the main loop
type Result<T> = std::result::Result<T, ServiceError>;
#[derive(Debug, Snafu)]
enum ServiceError {
    #[snafu(display("An I/O error occured while opening EC `{}`: {}", dev_path.display(), source))]
    OpenDev {
        dev_path: &'static Path,
        source: std::io::Error,
    },

    #[snafu(display("{}", source))]
    ECIO { source: ECError },

    #[snafu(display("{}", source))]
    ServiceConfigLoad {
        source: config::service::ServiceConfigLoadError,
    },

    #[snafu(display("{}", source))]
    ControlConfigLoad {
        source: config::nbfc_control::ControlConfigLoadError,
    },

    #[snafu(display("{}", source))]
    Sensor { source: temp::SensorError },
}
fn main() {
    pretty_env_logger::init();

    info!("Loading service configuration");

    let service_config = ServiceConfig::load_service_config()
        .context(ServiceConfigLoad)
        .unwrap_or_else(|e| {
            warn!("Failed to load service configuration: {}\nUsing default", e);
            ServiceConfig {
                auto: true,
                ..Default::default()
            }
        });
    let state = Rc::from(State::from(service_config));

    info!("Creating D-Bus connection");

    let dbus_conn = create_dbus_conn(Rc::clone(&state)).expect("Failed to create D-Bus connection");

    let fan_config = {
        if state.config.read().unwrap().trim().is_empty() {
            // Blocking the process until a valid configuration is provided.
            loop {
                dbus_conn.process(Duration::from_millis(1000)).unwrap();
                if !state.config.read().unwrap().is_empty() {
                    break;
                }
            }
        }
        info!(
            "Loading fan control configuration '{}'",
            &state.config.read().unwrap()
        );
        load_control_config(&*state.config.read().unwrap())
            .context(ControlConfigLoad)
            .unwrap()
    };

    unsafe {
        signal_hook::register(signal_hook::SIGTERM, || {
            info!("SIGTERM received. Exiting gracefully");
            cleaner();
            std::process::exit(0);
        })
        .unwrap();
    }

    let dev_path = state.ec_access_mode.read().unwrap().to_path();
    let ec_dev = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dev_path)
        .context(OpenDev { dev_path })
        .unwrap();
    let ec_manager = Rc::from(Mutex::new(ECManager::new(ec_dev)));
    ec_manager
        .lock()
        .unwrap()
        .refresh_control_config(fan_config)
        .context(ECIO {})
        .unwrap();

    // We catch this signal to save the config and to hook some calls.
    //XXX: UGLY CODE
    let d_state = Rc::clone(&state);
    let d_ec_m = Rc::clone(&ec_manager);
    dbus_conn
        .add_match(
            PropertiesPropertiesChanged::match_rule(Some(&BUS_NAME), Some(&DBUS_PATH)),
            move |props: PropertiesPropertiesChanged, _, _| {
                for (property, _val) in props.changed_properties {
                    match &*property {
                        "Config" => {
                            trace!(
                                "Swapping configuration to '{}'",
                                &*d_state.config.read().unwrap()
                            );
                            d_state.fans_speeds.write().unwrap().clear();
                            let conf =
                                load_control_config(&*d_state.config.read().unwrap()).unwrap();
                            d_ec_m.lock().unwrap().refresh_control_config(conf).unwrap();
                        }
                        _ => {}
                    }
                }
                trace!("Saving service configuration to disk");
                d_state.as_service_config().save().unwrap();
                true
            },
        )
        .unwrap();

    main_loop(ec_manager, dbus_conn, state)
        .map_err(|e| {
            cleaner();
            e
        })
        .unwrap()
}

fn main_loop<RW: Unpin + Read + Write + Seek>(
    ec_manager: Rc<Mutex<ECManager<RW>>>,
    dbus_conn: dbus_crate::blocking::LocalConnection,
    state: Rc<State>,
) -> Result<()> {
    loop {
        // We should normally use a timer to call the function at an interval but instead of losing time,
        // we treat the D-Bus requests.
        let timeout = ec_manager.lock().unwrap().poll_interval;
        dbus_conn.process(timeout).unwrap();

        let mut ec_manager = ec_manager.lock().unwrap();

        let temps = temp::Temperatures::get_temps().unwrap();
        temps.update_map(&mut *state.temps.write().unwrap());

        let critical = *state.critical.read().unwrap();
        let temp_lock = state.temps.read().unwrap();
        let temp_values = temp_lock.values();
        let temp: f64 = temp_values.clone().sum::<f64>() / temp_values.len() as f64;
        *state.critical.write().unwrap() = if !critical {
            temp as u8 >= ec_manager.critical_temperature
        } else {
            ec_manager.critical_temperature.saturating_sub(temp as u8) <= CRITICAL_INTERVAL
        };

        let mut fans_speeds = state.fans_speeds.write().unwrap();

        for i in 0..ec_manager.fan_configs.len() {
            fans_speeds.insert(
                ec_manager.fan_configs[i].name.to_owned(),
                ec_manager.read_fan_speed(i).context(ECIO {})?,
            );

            if *state.critical.read().unwrap() {
                ec_manager.write_fan_speed(i, 100.0).context(ECIO {})?;
            } else if !*state.auto.read().unwrap()
                && state.target_fans_speeds.read().unwrap().get(i).is_some()
            {
                ec_manager
                    .write_fan_speed(i, state.target_fans_speeds.read().unwrap()[i])
                    .context(ECIO {})?;
            } else if ec_manager.refresh_fan_threshold(temps.cpu_temp, i) {
                let value = ec_manager.fan_configs[i].thresholds[ec_manager.current_thr_indices[i]]
                    .fan_speed
                    .into();

                ec_manager.write_fan_speed(i, value).context(ECIO {})?;
            }
        }
    }
}
