/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::time::Duration;

use async_std::future;
use async_std::{channel, sync::Arc, task};
use futures::{try_join, StreamExt, TryFutureExt};
use loader::Loader;
use once_cell::sync::Lazy;
use signal_hook::consts::*;
use signal_hook_async_std::Signals;
use smol::Timer;
use snafu::{ResultExt, Snafu};

use ec_control::{ECManager, EcAccess, Event, ExternalEvent, RawPort, RW};
use nbfc_config as nbfc;
use temp::Temperatures;

use crate::ec_control::EcRW;

mod config;
mod constants;
mod ec_control;
mod loader;
mod state;
mod temp;

type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Debug, Snafu)]
enum ServiceError {
    #[snafu(display("An error occurred while opening EC: {}", source))]
    OpenDev {
        source: async_std::io::Error,
    },

    #[snafu(display("{}", source))]
    ECIO {
        source: ec_control::EcManagerError,
    },

    /*#[snafu(display("{}", source))]
    ServiceConfigLoad {
        source: config::service::ServiceConfigLoadError,
    },

    #[snafu(display("{}", source))]
    ControlConfigLoad {
        source: config::nbfc_control::ControlConfigLoadError,
    },*/
    #[snafu(display("{}", source))]
    Sensor {
        source: temp::SensorError,
    },

    #[snafu(display("{}", source))]
    DBus {
        source: zbus::Error,
    },

    ConfigErr {
        source: config::ConfigError,
    },

    #[snafu(display("{}", source))]
    Signal {
        source: std::io::Error,
    },

    #[snafu(display("{}", source))]
    ShutdownChannelRecv {
        source: async_std::channel::RecvError,
    },

    #[snafu(display("{}", source))]
    ShutdownChannelSend {
        source: async_std::channel::SendError<bool>,
    },
}

#[async_std::main]
async fn main() -> Result<()> {
    //TODO: Check errors
    let mut config = config::Config::load_config()
        .await
        .unwrap_or_else(|_| config::Config::default());

    let conn = zbus::Connection::system().await.context(DBus {})?;
    conn.request_name("com.musikid.fancy")
        .await
        .context(DBus {})?;
    let conn = Arc::from(conn);

    let mut temps = Temperatures::new(config.sensors.only.clone())
        .await
        .context(Sensor {})?;

    //TODO: Check errors
    let ec_device = EcAccess::from_mode(config.core.ec_access_mode)
        .or_else(|_| EcAccess::try_default())
        .await
        .context(OpenDev {})?;

    let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).context(Signal)?;
    let (shutdown_tx, shutdown_rx) = channel::bounded(1);
    let sig_handle = signals.handle();
    let signal_handler = task::spawn(async move {
        while let Some(sig) = signals.next().await {
            match sig {
                //TODO: Reload configuration?
                SIGHUP => {}
                SIGTERM | SIGINT | SIGQUIT => {
                    shutdown_tx.send(true).await.context(ShutdownChannelSend)?;
                    sig_handle.close();
                    break;
                }
                _ => {}
            }
        }

        Ok::<_, ServiceError>(())
    });

    let mut manager = ECManager::new(ec_device, Arc::clone(&conn));
    let event_sender = manager.create_sender();

    let loader = Loader::new(manager.create_sender()).await;
    conn.object_server()
        .at("/com/musikid/fancy/loader", loader)
        .await
        .context(DBus)?;

    let shutdown_recv = shutdown_rx.clone();
    let manager_task = task::spawn(async move {
        // We need to send the shutdown signal to the event loop
        try_join!(
            async { manager.event_handler().await.context(ECIO) },
            async { shutdown_recv.recv().await.context(ShutdownChannelRecv) }
        )?;

        manager.target_speeds().await.context(ECIO)
    });

    let shutdown_recv = shutdown_rx.clone();
    //TODO: Set interval?
    let temps_task = task::spawn(async move {
        loop {
            match future::timeout(Duration::from_millis(100), shutdown_recv.recv()).await {
                Ok(res) => {
                    if res.context(ShutdownChannelRecv)? {
                        break Ok::<_, ServiceError>(());
                    }
                }
                Err(_) => {
                    let temp = temps.get_temp().await.context(Sensor {})?;
                    event_sender
                        .send_event(Event::External(ExternalEvent::TempChange(temp)))
                        .await
                }
            }
        }
    });

    signal_handler.await?;
    let target_speeds = manager_task.await?;
    temps_task.await?;

    let loader_ref = conn
        .object_server()
        .interface::<_, Loader>("/com/musikid/fancy/loader")
        .await
        .context(DBus)?;
    let loader = loader_ref.get().await;

    if let Some(fan_config) = loader.current_config.as_ref().map(|t| t.0.clone()) {
        config.fan_config.selected_fan_configuration = fan_config;
    }
    if !target_speeds.is_empty() {
        config.fan_config.target_speeds = target_speeds;
    }
    //config.core.ec_access_mode = ec_device.mode();

    config.save_config().await.context(ConfigErr)?;

    Ok(())
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::fs::{read_dir, OpenOptions};
    use std::io::Read;
    use std::path::PathBuf;

    use rayon::prelude::*;
    use rstest::fixture;

    use nbfc_config::{FanControlConfigV2, XmlFanControlConfigV2};

    #[fixture]
    #[once]
    pub fn parsed_configs() -> Vec<FanControlConfigV2> {
        let paths: Vec<PathBuf> = read_dir("nbfc_configs/Configs")
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        paths
            .par_iter()
            .map(|path| {
                let mut file = OpenOptions::new().read(true).open(path).unwrap();

                let mut buf = String::with_capacity(4096);
                file.read_to_string(&mut buf).unwrap();
                buf
            })
            .map(|s| {
                //TODO: Other extensions
                quick_xml::de::from_str::<XmlFanControlConfigV2>(&s)
                    .unwrap()
                    .into()
            })
            .collect()
    }
}
