/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::message::SignalArgs;
use dbus::strings::{BusName, Path as DBusPath};
use log::{info, trace};
use once_cell::sync::Lazy;
use snafu::{ResultExt, Snafu};

use std::path::Path;
use std::rc::Rc;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use std::time::Duration;

mod bus;
mod config;
mod constants;
mod ec_control;
mod nbfc;
mod state;
mod temp;

use bus::connection::create_dbus_conn;
use config::{nbfc_control::load_control_config, service::ServiceConfig};
use constants::OBJ_PATH_STR;
use ec_control::{ECError, ECManager, RawPort, RW};
use state::State;

const CRITICAL_INTERVAL: u8 = 10;

const BUS_NAME_STR: &str = "com.musikid.fancy";
static BUS_NAME: Lazy<BusName> = Lazy::new(|| BusName::new(BUS_NAME_STR).unwrap());
static DBUS_PATH: Lazy<DBusPath> = Lazy::new(|| DBusPath::new(OBJ_PATH_STR).unwrap());

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
    Sensor { source: temp::SensorError },

    #[snafu(display("{}", source))]
    DBus { source: dbus::Error },

    #[snafu(display("{}", source))]
    Signal { source: std::io::Error },
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    info!("Loading service configuration");

    //TODO: Treat errors
    let service_config = ServiceConfig::load_service_config().unwrap_or_else(|_| {
        info!(
            "Failed to load service configuration
            Using default values"
        );
        ServiceConfig {
            auto: true,
            ..Default::default()
        }
    });

    // We have to check if it's /dev/port because we have to "wrap" the file in this case.
    let is_raw_port =
        service_config.ec_access_mode == crate::config::service::ECAccessMode::RawPort;

    let state = Rc::from(State::from(service_config));
    let dbus_conn = create_dbus_conn(Rc::clone(&state)).expect("Failed to create D-Bus connection");

    let fan_config = get_fan_config(Rc::clone(&state), &dbus_conn);

    let dev_path = state.ec_access_mode.borrow().to_path();

    let ec_dev = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dev_path)
        .context(OpenDev { dev_path })?;

    // XXX: Sorry...
    let ec_dev = if is_raw_port {
        Box::from(RawPort::from(ec_dev)) as Box<dyn RW>
    } else {
        Box::from(ec_dev) as Box<dyn RW>
    };

    let ec_manager = ECManager::new(ec_dev);
    let ec_manager = Rc::from(Mutex::new(ec_manager));
    ec_manager
        .lock()
        .unwrap()
        .refresh_control_config(fan_config)
        .context(ECIO {})?;

    *state.ec_access_mode.borrow_mut() = crate::config::service::ECAccessMode::from(dev_path);

    {
        // We have to clone the references to move them to the closure.
        let state = Rc::clone(&state);
        let ec_manager = Rc::clone(&ec_manager);
        // We catch the signal when a property changed to save the config and to hook some calls.
        //XXX: VERY UGLY CODE
        dbus_conn
            .add_match(
                PropertiesPropertiesChanged::match_rule(Some(&BUS_NAME), Some(&DBUS_PATH)),
                move |props: PropertiesPropertiesChanged, _, _| {
                    for (property, _val) in props.changed_properties {
                        match &*property {
                            "Config" => {
                                let config = state.config.borrow();
                                trace!("Swapping configuration to '{}'", &*config);

                                let mut target_fans_speeds = state.fans_speeds.borrow_mut();
                                target_fans_speeds.clear();

                                let conf = load_control_config(&*config).unwrap();

                                ec_manager
                                    .lock()
                                    .unwrap()
                                    .refresh_control_config(conf)
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }

                    trace!("Saving service configuration");
                    state.as_service_config().save().unwrap();
                    true
                },
            )
            .context(DBus {})?;
    }

    main_loop(ec_manager, dbus_conn, state)
}

/// Get the fan configuration in the service config if applicable, else blocks the process until a
/// valid one is provided.
fn get_fan_config(
    state: Rc<State>,
    dbus_conn: &dbus::blocking::LocalConnection,
) -> nbfc::FanControlConfigV2 {
    let mut fan_config = state.config.borrow();
    if fan_config.trim().is_empty() {
        // Blocking the process until a valid configuration is provided.
        loop {
            dbus_conn.process(Duration::from_millis(1000)).unwrap();
            fan_config = state.config.borrow();
            if !fan_config.trim().is_empty() {
                break;
            }
        }
    }

    info!("Loading fan control configuration '{}'", &fan_config);

    load_control_config(&*fan_config).unwrap()
}

fn main_loop<T: RW>(
    ec_manager: Rc<Mutex<ECManager<T>>>,
    dbus_conn: dbus::blocking::LocalConnection,
    state: Rc<State>,
) -> Result<()> {
    let signal_received = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&signal_received))
        .context(Signal {})?;

    loop {
        if signal_received.load(std::sync::atomic::Ordering::Relaxed) {
            let mut ec_manager = ec_manager.lock().unwrap();
            return ec_manager.reset_ec(true).context(ECIO {});
        }
        // We should normally use a timer (or convert service to async?) to call the function at an interval but instead of losing time,
        // we treat the D-Bus requests.
        let timeout = {
            let t = ec_manager.lock().unwrap().poll_interval;
            if t > Duration::from_nanos(0) {
                t
            } else {
                Duration::from_millis(10)
            }
        };
        dbus_conn.process(timeout).context(DBus {})?;

        let mut ec_manager = ec_manager.lock().unwrap();

        // TODO: Find a way to optimize that
        let current_temps = temp::Temperatures::get_temps().context(Sensor {})?;
        let mut state_temps = state.temps.borrow_mut();
        current_temps.update_map(&mut state_temps);

        let temp_values = state_temps.values();
        let temp: f64 = temp_values.clone().sum::<f64>() / temp_values.len() as f64;

        let critical_now = *state.critical.borrow();
        let mut critical_temp = state.critical.borrow_mut();

        *critical_temp = if !critical_now {
            temp as u8 >= ec_manager.critical_temperature
        } else {
            ec_manager.critical_temperature.saturating_sub(temp as u8) <= CRITICAL_INTERVAL
        };

        let mut fans_speeds = state.fans_speeds.borrow_mut();

        for i in 0..ec_manager.fan_configs.len() {
            fans_speeds.insert(
                ec_manager.fan_configs[i].name.to_owned(),
                ec_manager.read_fan_speed(i).context(ECIO {})?,
            );

            // If there is a target fan speed set by the user
            let user_defined_speed =
                !*state.auto.borrow() && state.target_fans_speeds.borrow().get(i).is_some();

            if *critical_temp {
                ec_manager.write_fan_speed(i, 100.0).context(ECIO {})?;
            } else if user_defined_speed {
                ec_manager
                    .write_fan_speed(i, state.target_fans_speeds.borrow()[i])
                    .context(ECIO {})?;
            }
            // If the function returns `true`, the threshold has changed.
            // Else, there is nothing to change.
            else if ec_manager.refresh_fan_threshold(current_temps.cpu_temp, i) {
                let threshold = ec_manager.fan_configs[i].current_threshold;
                let value = ec_manager.fan_configs[i].thresholds[threshold]
                    .fan_speed
                    .into();

                ec_manager.write_fan_speed(i, value).context(ECIO {})?;
            }
        }
    }
}
