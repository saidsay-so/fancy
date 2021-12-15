/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use async_std::sync::{Arc, Mutex, RwLock};
use snafu::{ResultExt, Snafu};
use zbus::{dbus_interface, ObjectServer, SignalContext};

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::time::Duration;
use crate::ec_control::EcRW;

use super::read::ECReader;
use super::write::ECWriter;
use super::RW;
use crate::nbfc::*;

#[derive(Debug, Snafu)]
pub(crate) enum ECError {
    #[snafu(display("An I/O error occured with the writer: {}", source))]
    Writer { source: std::io::Error },

    #[snafu(display("An I/O error occured with the reader: {}", source))]
    Reader { source: std::io::Error },
}

type Result<T = ()> = std::result::Result<T, ECError>;

/// Holds information on a fan.
#[derive(Debug)]
pub(crate) struct Fan {
    pub name: String,
    pub thresholds: Vec<TemperatureThreshold>,
    pub current_threshold: usize,
    pub fan_speed: Arc<RwLock<f64>>,
    pub target_speed: Arc<RwLock<f64>>,
}

#[dbus_interface(name = "com.musikid.fancy.Fan")]
impl Fan {
    #[dbus_interface(property, name = "Name")]
    fn name(&self) -> String {
        self.name.clone()
    }

    #[dbus_interface(property, name = "FanSpeed")]
    async fn fan_speed(&self) -> f64 {
        *self.fan_speed.read().await
    }

    #[dbus_interface(property, name = "TargetSpeed")]
    async fn target_speed(&self) -> f64 {
        *self.target_speed.read().await
    }

    #[dbus_interface(property, name = "TargetSpeed")]
    async fn set_target_speed(&mut self, target: f64) {
        *self.target_speed.write().await = target;
    }

    // #[dbus_interface(property, name = "CurrentThreshold")]
    // fn current_threshold(&self) -> u8 {
    //     self.current_threshold as u8
    // }
}

/// Manages accesses to the EC.
#[derive(Debug)]
pub(crate) struct ECManager<T: EcRW> {
    pub poll_interval: Duration,
    pub fan_configs: Vec<Fan>,
    pub critical_temperature: u8,
    reader: ECReader<T>,
    writer: ECWriter<T>,
}

impl<T: EcRW> ECManager<T> {
    pub fn new(ec_device: T) -> Self {
        let ec_device = Arc::from(Mutex::from(ec_device));

        ECManager {
            poll_interval: Duration::ZERO,
            fan_configs: Vec::new(),
            critical_temperature: 0,
            writer: ECWriter::new(Arc::clone(&ec_device)),
            reader: ECReader::new(Arc::clone(&ec_device)),
        }
    }

    /// Refresh the fan(s) configuration and initialize the writer according to this config.
    pub async fn refresh_control_config(&mut self, c: FanControlConfigV2) -> Result {
        self.fan_configs = c
            .fan_configurations
            .iter()
            .scan(0, |acc, f| {
                *acc += 1;
                Some(Fan {
                    name: f
                        .fan_display_name
                        .to_owned()
                        .unwrap_or(format!("Fan #{}", acc)),
                    thresholds: f.temperature_thresholds.to_owned(),
                    current_threshold: 0,
                    fan_speed: Arc::from(RwLock::from(0.0)),
                    target_speed: Arc::from(RwLock::from(0.0)),
                })
            })
            .collect();

        self.critical_temperature = c.critical_temperature;
        self.poll_interval = Duration::from_millis(c.ec_poll_interval);

        self.fan_configs
            .iter_mut()
            .for_each(|c| c.thresholds.sort());

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

    /// Refresh the index of the current fan threshold according to the temperature (if necessary).
    /// Returns false if the threshold didn't need change.
    ///
    /// # Panics
    ///
    /// Panics if the temperature cannot be converted to `u8`.
    /// Panics if the thresholds has no elements.
    pub fn refresh_fan_threshold(&mut self, temp: f64, fan_index: usize) -> bool {
        let temp = temp as u8;
        let fan_config = &mut self.fan_configs[fan_index];
        let thresholds = &fan_config.thresholds;
        let current = &mut fan_config.current_threshold;

        if temp >= thresholds.last().unwrap().up_threshold {
            *current = thresholds.len() - 1;
        } else if temp >= thresholds[*current].down_threshold
            && temp <= thresholds[*current].up_threshold
        {
            return false;
        } else if matches!(thresholds.iter().find(|t| t.down_threshold != 0), Some(thr) if temp <= thr.down_threshold)
            || thresholds.len() == 1
        {
            *current = 0;
        } else if let Ok(i) = thresholds.binary_search_by(|el| match el {
            _t if _t.down_threshold > temp => Ordering::Greater,
            _t if _t.up_threshold < temp => Ordering::Less,
            _ => Ordering::Equal,
        }) {
            *current = i;
        }

        true
    }

    /// Write the speed percent to the EC for the fan specified by `fan_index`.
    pub async fn write_fan_speed(&mut self, fan_index: usize, speed_percent: f64) -> Result {
        self.writer
            .write_speed_percent(fan_index, speed_percent)
            .await
            .context(Writer {})
    }

    /// Reset the EC, including non-required registers when `reset_all` is true.
    pub async fn reset_ec(&mut self, reset_all: bool) -> Result {
        self.writer.reset(reset_all).await.context(Writer {})
    }

    /// Read the speed percent from the EC for the fan specified by `fan_index`.
    pub async fn read_fan_speed(&mut self, fan_index: usize) -> Result<f64> {
        self.reader
            .read_speed_percent(fan_index)
            .await
            .context(Reader {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::{
        io::{Cursor, Read, Seek},
        task::block_on,
    };
    use once_cell::sync::Lazy;

    static CONFIGS_PARSED: Lazy<Vec<FanControlConfigV2>> = Lazy::new(|| {
        use std::io::Read;

        std::fs::read_dir("nbfc_configs/Configs")
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| std::fs::File::open(e.path()).unwrap())
            .map(|mut e| {
                let mut buf = String::new();
                e.read_to_string(&mut buf).unwrap();
                buf
            })
            .map(|e| {
                quick_xml::de::from_str::<XmlFanControlConfigV2>(&e)
                    .unwrap()
                    .into()
            })
            .collect()
    });

    #[test]
    fn refresh_config() {
        use is_sorted::IsSorted;
        CONFIGS_PARSED.iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0u8; 256]);

                let mut manager = ECManager::new(ec);

                manager.refresh_control_config(c.clone()).await.unwrap();

                assert!(manager.critical_temperature == c.critical_temperature as u8);
                assert!(manager
                    .fan_configs
                    .iter_mut()
                    .all(|f| { IsSorted::is_sorted(&mut f.thresholds.iter()) }));
                assert!(manager.fan_configs.iter().all(|f| f.current_threshold == 0));
            })
        });
    }

    #[test]
    fn select_threshold() {
        CONFIGS_PARSED.iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0u8; 256]);

                let mut manager = ECManager::new(ec);

                manager.refresh_control_config(c.clone()).await.unwrap();

                for i in 0..c.fan_configurations.len() {
                    let thresholds = &c.fan_configurations[i].temperature_thresholds;

                    let very_high_temperature = 90.0;
                    manager.refresh_fan_threshold(very_high_temperature, i);
                    assert!(manager.fan_configs[i].current_threshold == thresholds.len() - 1);

                    let very_low_temperature = 20.0;
                    manager.refresh_fan_threshold(very_low_temperature, i);
                    assert!(
                        manager.fan_configs[i].current_threshold == 0
                            || manager.fan_configs[i].current_threshold == 1
                    );

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
