/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cmp::Ordering;
use std::time::Duration;

use async_std::channel::{self, Receiver, Sender};
use async_std::future;
use async_std::sync::{Arc, Mutex};
use snafu::{ResultExt, Snafu};
use zbus::zvariant::OwnedObjectPath;
use zbus::{dbus_interface, Connection};

use crate::ec_control::EcRW;
use crate::nbfc::*;

use super::read::ECReader;
use super::write::ECWriter;

const CRITICAL_INTERVAL: u8 = 10;
const INFINITE_DURATION: Duration = Duration::from_secs(u64::MAX);

#[derive(Debug, Snafu)]
pub(crate) enum EcManagerError {
    #[snafu(display("An I/O error occured with the writer: {}", source))]
    Writer { source: std::io::Error },

    #[snafu(display("An I/O error occured with the reader: {}", source))]
    Reader { source: std::io::Error },

    #[snafu(display("Failed to receive message: {}", source))]
    RecvEvent {
        source: async_std::channel::RecvError,
    },

    #[snafu(display("Error while modifying object server: {}", source))]
    Dbus { source: zbus::Error },
}

type Result<T = ()> = std::result::Result<T, EcManagerError>;

/// Holds information on a fan.
#[derive(Debug)]
struct Fan {
    pub name: String,
    pub index: usize,
    pub ev_sender: EventSender,
    pub thresholds: Vec<TemperatureThreshold>,
    pub current_threshold: usize,
    pub auto: bool,
    pub fan_speed: f64,
    pub target_speed: f64,
}

impl Fan {
    /// Refresh the index of the current fan threshold according to the temperature (if necessary).
    /// Returns None if the threshold didn't need change, else the speed.
    ///
    /// # Panics
    ///
    /// Panics if the temperature cannot be converted to `u8`.
    /// Panics if there is no threshold.
    pub fn refresh_fan_threshold(&mut self, temp: f64) -> Option<f64> {
        let temp = temp as u8;
        let thresholds = &self.thresholds;
        let current = self.current_threshold();

        self.current_threshold = if temp >= thresholds.last().unwrap().up_threshold {
            thresholds.len() - 1
        } else if temp >= current.down_threshold && temp <= current.up_threshold {
            return None;
        } else if matches!(thresholds.iter().find(|t| t.down_threshold != 0), Some(thr) if temp <= thr.down_threshold)
            || thresholds.len() == 1
        {
            0
        } else if let Ok(i) = thresholds.binary_search_by(|el| match el {
            _t if _t.down_threshold > temp => Ordering::Greater,
            _t if _t.up_threshold < temp => Ordering::Less,
            _ => Ordering::Equal,
        }) {
            i
        } else {
            return None;
        };

        Some(self.current_threshold().fan_speed.into())
    }

    //TODO: Maybe turn as property?
    fn current_threshold(&self) -> TemperatureThreshold {
        self.thresholds[self.current_threshold].clone()
    }
}

#[dbus_interface(name = "com.musikid.fancy.Fan")]
impl Fan {
    #[dbus_interface(property, name = "Name")]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[dbus_interface(property, name = "FanSpeed")]
    async fn fan_speed(&self) -> f64 {
        //TODO: Send only in immediate mode
        self.ev_sender
            .send_event(Event::Manager(ManagerEvent::QuerySpeed(self.index)))
            .await;
        self.fan_speed
    }

    #[dbus_interface(property, name = "Auto")]
    fn auto(&self) -> bool {
        self.auto
    }

    #[dbus_interface(property, name = "Auto")]
    async fn set_auto(&mut self, auto: bool) {
        self.auto = auto;
        self.ev_sender
            .send_event(Event::Manager(ManagerEvent::Auto(self.index)))
            .await;
    }

    #[dbus_interface(property, name = "TargetSpeed")]
    fn target_speed(&self) -> f64 {
        self.target_speed
    }

    #[dbus_interface(property, name = "TargetSpeed")]
    async fn set_target_speed(&mut self, target: f64) {
        self.target_speed = target;
        self.ev_sender
            .send_event(Event::Manager(ManagerEvent::TargetSpeed(self.index)))
            .await;
    }

    // #[dbus_interface(property, name = "CurrentThreshold")]
    // fn current_threshold(&self) -> u8 {
    //     self.current_threshold as u8
    // }
}

#[derive(Clone, Debug)]
struct FansInfo {
    count: usize,
    paths: Vec<OwnedObjectPath>,
}

#[dbus_interface(name = "com.musikid.fancy.Fans")]
impl FansInfo {
    #[dbus_interface(property, name = "Count")]
    fn count(&self) -> u64 {
        self.count as u64
    }

    #[dbus_interface(property, name = "Paths")]
    fn paths(&self) -> Vec<OwnedObjectPath> {
        self.paths.clone()
    }
}

#[derive(Debug)]
pub(crate) struct EventSender(Sender<Event>);

impl EventSender {
    pub async fn send_event(&self, ev: Event) {
        self.0.send(ev).await.unwrap()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ManagerEvent {
    TargetSpeed(usize),
    Auto(usize),
    QuerySpeed(usize),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Event {
    Manager(ManagerEvent),
    External(ExternalEvent),
}

/// Events external to the manager
#[derive(Debug, PartialEq)]
pub(crate) enum ExternalEvent {
    TempChange(f64),
    RefreshConfig(FanControlConfigV2),
    Shutdown,
}

/// Manages accesses to the EC.
#[derive(Debug)]
pub(crate) struct ECManager<T: EcRW> {
    conn: Arc<Connection>,
    fans_info: FansInfo,
    /// Interval at which information are refreshed.
    /// `Duration::ZERO` means that information is updated when needed.
    poll_interval: Duration,
    immediate: bool,
    critical_temperature: u8,
    current_temperature: f64,
    ev_sender: Sender<Event>,
    ev_receiver: Receiver<Event>,
    critical: bool,
    reader: ECReader<T>,
    writer: ECWriter<T>,
}

impl<T: EcRW> ECManager<T> {
    pub fn new(ec_device: T, conn: Arc<Connection>) -> Self {
        let ec_device = Arc::from(Mutex::from(ec_device));
        let (ev_sender, ev_receiver) = channel::unbounded();

        ECManager {
            conn,
            poll_interval: Duration::from_secs(u64::MAX),
            fans_info: FansInfo {
                count: 0,
                paths: Vec::new(),
            },
            immediate: false,
            ev_receiver,
            ev_sender,
            critical_temperature: 0,
            current_temperature: 100.0,
            critical: true,
            writer: ECWriter::new(Arc::clone(&ec_device)),
            reader: ECReader::new(Arc::clone(&ec_device)),
        }
    }

    pub fn create_sender(&self) -> EventSender {
        EventSender(self.ev_sender.clone())
    }

    /// Refresh the fan(s) configuration and initialize the writer according to this config.
    async fn refresh_control_config(&mut self, c: FanControlConfigV2) -> Result {
        const FANS_PATH: &str = "/com/musikid/fancy/fans";

        let obj = self.conn.object_server();

        // We don't return an error if the interface wasn't instantiated
        obj.remove::<FansInfo, _>(FANS_PATH)
            .await
            .or_else(|e| match e {
                zbus::Error::InterfaceNotFound => Ok(false),
                _ => Err(e),
            })
            .context(Dbus {})?;

        let fans: Vec<Fan> = c
            .fan_configurations
            .iter()
            .scan(0, |index, f| {
                *index += 1;
                let mut thresholds = f.temperature_thresholds.to_owned();
                thresholds.sort();
                Some(Fan {
                    name: f
                        .fan_display_name
                        .to_owned()
                        .unwrap_or_else(|| format!("Fan #{}", index)),
                    index: *index - 1,
                    thresholds,
                    ev_sender: self.create_sender(),
                    auto: true,
                    current_threshold: 0,
                    fan_speed: 0.0,
                    target_speed: 100.0,
                })
            })
            .collect();
        let count = fans.len();

        let mut paths = Vec::new();

        for (i, fan) in fans.into_iter().enumerate() {
            let path = format!("/com/musikid/fancy/fans/{}", i);
            paths.push(OwnedObjectPath::try_from(&*path).unwrap());
            obj.remove::<Fan, _>(&*path)
                .await
                .or_else(|e| match e {
                    zbus::Error::InterfaceNotFound => Ok(false),
                    _ => Err(e),
                })
                .context(Dbus {})?;
            obj.at(path, fan).await.context(Dbus {})?;
        }

        self.fans_info = FansInfo { count, paths };

        obj.at(FANS_PATH, self.fans_info.clone())
            .await
            .context(Dbus {})?;

        self.critical_temperature = c.critical_temperature;
        self.critical = true;
        self.poll_interval = Duration::from_millis(c.ec_poll_interval);

        self.reader
            .refresh_config(c.read_write_words, &c.fan_configurations);

        self.writer
            .refresh_config(
                c.read_write_words,
                c.register_write_configurations,
                &c.fan_configurations,
            )
            .await
            .context(Writer {})
    }

    pub async fn event_handler(&mut self) -> Result {
        if self.fans_info.count == 0 {
            loop {
                match self.ev_receiver.recv().await.context(RecvEvent)? {
                    Event::External(ExternalEvent::RefreshConfig(config)) => {
                        self.refresh_control_config(config).await?;
                        break;
                    }
                    Event::External(ExternalEvent::Shutdown) => return Ok(()),
                    _ => {}
                }
            }
        }

        loop {
            let timeout = if self.immediate {
                INFINITE_DURATION
            } else {
                self.poll_interval
            };

            if let Ok(ev_res) = future::timeout(timeout, self.ev_receiver.recv()).await {
                match ev_res.context(RecvEvent {})? {
                    Event::Manager(ManagerEvent::Auto(i))
                    | Event::Manager(ManagerEvent::TargetSpeed(i)) => {
                        self.write_fan_speed(i).await?;
                    }
                    Event::Manager(ManagerEvent::QuerySpeed(i)) => self.read_fan_speed(i).await?,
                    // External events
                    Event::External(ExternalEvent::RefreshConfig(config)) => {
                        self.refresh_control_config(config).await?;
                    }
                    Event::External(ExternalEvent::TempChange(temp)) => {
                        self.update_temp(temp);
                        self.write_speeds().await?;
                    }
                    Event::External(ExternalEvent::Shutdown) => {
                        self.reset_ec(false).await?;
                        break Ok(());
                    }
                }
            }

            // Timeout with poll interval, so we read/write speeds

            self.read_speeds().await?;
            self.write_speeds().await?;
        }
    }

    async fn read_speeds(&self) -> Result {
        for i in 0..self.fans_info.count {
            self.read_fan_speed(i).await?;
        }

        Ok(())
    }

    async fn write_speeds(&self) -> Result {
        for i in 0..self.fans_info.count {
            self.write_fan_speed(i).await?;
        }

        Ok(())
    }

    pub async fn target_speeds(&self) -> Result<Vec<f64>> {
        let paths = &self.fans_info.paths;
        let mut speeds = Vec::with_capacity(self.fans_info.count);

        for fan_path in paths.iter() {
            let fan_iface_ref = self
                .conn
                .object_server()
                .interface::<_, Fan>(fan_path)
                .await
                .context(Dbus)?;
            let fan = fan_iface_ref.get().await;
            speeds.push(fan.target_speed);
        }

        Ok(speeds)
    }

    fn update_temp(&mut self, temp: f64) {
        self.critical = if self.critical {
            self.critical_temperature.saturating_sub(temp as u8) <= CRITICAL_INTERVAL
        } else {
            temp as u8 >= self.critical_temperature
        };

        self.current_temperature = temp;
    }

    /// Write the speed percent to the EC for the fan specified by `fan_index`.
    async fn write_fan_speed(&self, fan_index: usize) -> Result {
        let speed = {
            let fan_iface_ref = self
                .conn
                .object_server()
                .interface::<_, Fan>(&*self.fans_info.paths[fan_index])
                .await
                .context(Dbus)?;
            let mut fan = fan_iface_ref.get_mut().await;

            if self.critical {
                Some(100.0)
            } else if fan.auto {
                fan.refresh_fan_threshold(self.current_temperature)
            } else {
                // We want to write the speed only if it has changed.
                let speed = fan.target_speed;
                ((fan.fan_speed - speed).abs() > f64::EPSILON).then(|| speed)
            }
        };

        if let Some(speed) = speed {
            self.writer
                .write_speed_percent(fan_index, speed)
                .await
                .context(Writer {})?;

            if self.immediate {
                self.read_fan_speed(fan_index).await?;
            }
        }

        Ok(())
    }

    /// Reset the EC, including non-required registers when `reset_all` is true.
    async fn reset_ec(&self, reset_all: bool) -> Result {
        self.writer.reset(reset_all).await.context(Writer {})
    }

    /// Read the speed percent from the EC for the fan specified by `fan_index`.
    async fn read_fan_speed(&self, fan_index: usize) -> Result {
        let speed = self
            .reader
            .read_speed_percent(fan_index)
            .await
            .context(Reader {})?;
        let iface_ref = self
            .conn
            .object_server()
            .interface::<_, Fan>(self.fans_info.paths[fan_index].clone())
            .await
            .context(Dbus {})?;
        let mut fan = iface_ref.get_mut().await;

        fan.fan_speed = speed;
        fan.fan_speed_changed(iface_ref.signal_context())
            .await
            .context(Dbus {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::parsed_configs;
    use async_std::{
        io::Cursor,
        task::{self, block_on},
    };
    use rayon::prelude::*;
    use rstest::rstest;
    use smol::stream::StreamExt;
    use std::sync::Arc;
    use zbus::MessageStream;

    #[rstest]
    fn refresh_config(parsed_configs: &Vec<FanControlConfigV2>) {
        use is_sorted::IsSorted;

        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let conn = Arc::from(zbus::Connection::session().await.unwrap());
                let ec = Cursor::new(vec![0u8; 256]);

                let mut manager = ECManager::new(ec, Arc::clone(&conn));

                manager.refresh_control_config(c.clone()).await.unwrap();

                assert_eq!(manager.critical_temperature, c.critical_temperature);
                for fan_path in manager.fans_info.paths {
                    let iface_ref = conn
                        .object_server()
                        .interface::<_, Fan>(fan_path)
                        .await
                        .unwrap();
                    let iface = iface_ref.get_mut().await;
                    assert!(IsSorted::is_sorted(&mut iface.thresholds.iter()));
                    assert_eq!(iface.current_threshold, 0);
                }

                // Connection
                let obj = conn
                    .object_server()
                    .interface::<_, FansInfo>("/com/musikid/fancy/fans")
                    .await
                    .unwrap();
                let fans_info = obj.get().await;
                let expected_paths: Vec<OwnedObjectPath> = (0..c.fan_configurations.len())
                    .map(|i| {
                        OwnedObjectPath::try_from(format!("/com/musikid/fancy/fans/{}", i)).unwrap()
                    })
                    .collect();
                assert_eq!(c.fan_configurations.len(), fans_info.count() as usize);
                assert_eq!(expected_paths, fans_info.paths());
            })
        });
    }

    #[rstest]
    fn emit_events(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().enumerate().for_each(|(i, c)| {
            block_on(async {
                let conn = Arc::from(zbus::Connection::session().await.unwrap());
                let ec = Cursor::new(vec![0u8; 256]);

                let mut manager = ECManager::new(ec, Arc::clone(&conn));
                manager.immediate = true;
                let service_name = format!("com.musikid.fancy.test{}", i);

                manager.refresh_control_config(c.clone()).await.unwrap();

                conn.request_name(service_name.clone()).await.unwrap();

                let paths = manager.fans_info.paths.clone();

                for (i, fan_path) in paths.into_iter().enumerate() {
                    {
                        let obj = conn
                            .object_server()
                            .interface::<_, Fan>(fan_path.clone())
                            .await
                            .unwrap();
                        let mut fan = obj.get_mut().await;

                        // Target speed
                        fan.set_target_speed(100.0).await;
                        assert_eq!(
                            Event::Manager(ManagerEvent::TargetSpeed(i)),
                            manager.ev_receiver.recv().await.unwrap()
                        );
                        assert_eq!(fan.target_speed, 100.0);

                        fan.set_auto(false).await;
                        assert_eq!(
                            Event::Manager(ManagerEvent::Auto(i)),
                            manager.ev_receiver.recv().await.unwrap()
                        );
                        assert_eq!(fan.auto, false);
                    }

                    manager.write_fan_speed(i).await.unwrap();
                    assert_eq!(
                        Event::Manager(ManagerEvent::QuerySpeed(i)),
                        manager.ev_receiver.recv().await.unwrap()
                    );

                    let (confirm_tx, conf_rx) = channel::bounded(1);

                    let match_rule = format!(
                        "type='signal',sender='{}',member='PropertiesChanged',path='{}'",
                        &*service_name,
                        fan_path.as_str()
                    );
                    let monitor_conn = Connection::session().await.unwrap();
                    monitor_conn
                        .call_method(
                            Some("org.freedesktop.DBus"),
                            "/org/freedesktop/DBus",
                            Some("org.freedesktop.DBus.Monitoring"),
                            "BecomeMonitor",
                            &(&[match_rule] as &[_], 0u32),
                        )
                        .await
                        .unwrap();
                    let mut monitor_stream = MessageStream::from(monitor_conn);
                    task::spawn(async move {
                        let res = monitor_stream.try_next().await;
                        if let Err(e) = res {
                            panic!("{}", e);
                        }
                        confirm_tx.send(true).await.unwrap();
                    });

                    manager.read_fan_speed(i).await.unwrap();

                    assert!(conf_rx.recv().await.unwrap());

                    assert_eq!(
                        Event::Manager(ManagerEvent::QuerySpeed(i)),
                        manager.ev_receiver.recv().await.unwrap()
                    );
                }
            });
        })
    }

    #[rstest]
    fn select_threshold(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0u8; 256]);
                let conn = Arc::from(zbus::Connection::session().await.unwrap());

                let mut manager = ECManager::new(ec, Arc::clone(&conn));

                manager.refresh_control_config(c.clone()).await.unwrap();

                for i in 0..c.fan_configurations.len() {
                    /*           let thresholds = &c.fan_configurations[i].temperature_thresholds;*/

                    /*let very_high_temperature = 120.0;*/
                    /*manager.fan_configs[i].refresh_fan_threshold(very_high_temperature);*/
                    /*assert_eq!(*/
                    /*manager.fan_configs[i].current_threshold.load(Relaxed),*/
                    /*thresholds.len() - 1*/
                    /*);*/

                    /*let very_low_temperature = 20.0;*/
                    /*manager.fan_configs[i].refresh_fan_threshold(very_low_temperature);*/
                    /*assert!(*/
                    /*(0..=1).contains(&manager.fan_configs[i].current_threshold.load(Relaxed))*/
                    /*);*/

                    // TODO: Find a way to test for other thresholds
                    // let mut rng = rand::thread_rng();

                    // for t in 50..80 {
                    //     println!("t°:{}", t);
                    //     // let random_temp = rng.gen_range(40.0, 80.0);

                    //     manager.refresh_fan_threshold(t as f64, i);

                    //     let thr = manager.current_thr_indices[i];
                    //     println!("thr:{}", thr);
                    //     let excepted_thr = match manager.fan_configurations[i]
                    //         .temperature_thresholds
                    //         .binary_search_by(|el| match el {
                    //             tmp if tmp.down_threshold >= t as u8 => Ordering::Greater,
                    //             tmp if tmp.up_threshold <= t as u8 => Ordering::Less,
                    //             _ => Ordering::Equal,
                    //         }) {
                    //         Ok(ei) => ei,
                    //         Err(ei) => ei - 1,
                    //     };
                    //     println!("ethr:{}", excepted_thr);

                    //     assert!(thr == excepted_thr);
                    // }
                }
            })
        });
    }

    // #[test]
    // fn requests() {

    // }
}
