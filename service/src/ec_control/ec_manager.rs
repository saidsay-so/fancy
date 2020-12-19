/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use snafu::{ResultExt, Snafu};

use std::cell::RefCell;
use std::rc::Rc;
use std::{cmp::Ordering, time::Duration};

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

type Result<T> = std::result::Result<T, ECError>;

#[derive(Debug)]
/// Lighter struct than `FanConfiguration` which holds useful information
/// not used by the writer or the reader.
pub(crate) struct FanConfig {
    pub name: String,
    pub thresholds: Vec<TemperatureThreshold>,
}

/// Manages accesses to the EC.
#[derive(Debug)]
pub(crate) struct ECManager<T: RW> {
    pub poll_interval: Duration,
    pub fan_configs: Vec<FanConfig>,
    pub critical_temperature: u8,
    /// We store the current threshold(s) by using indices.
    pub current_thr_indices: Vec<usize>,
    reader: ECReader<T>,
    writer: ECWriter<T>,
}

impl<T: RW> ECManager<T> {
    pub fn new(ec_device: T) -> Self {
        let ec_device = Rc::from(RefCell::from(ec_device));

        ECManager {
            poll_interval: Duration::from_nanos(0),
            current_thr_indices: Vec::new(),
            fan_configs: Vec::new(),
            critical_temperature: 0,
            writer: ECWriter::new(Rc::clone(&ec_device)),
            reader: ECReader::new(Rc::clone(&ec_device)),
        }
    }

    /// Refresh the `FanControlConfigV2` and initialize the writer according to this config.
    pub fn refresh_control_config(&mut self, c: FanControlConfigV2) -> Result<()> {
        self.fan_configs = c
            .fan_configurations
            .iter()
            .scan(0u8, |acc, f| {
                *acc += 1;
                Some(FanConfig {
                    name: f
                        .fan_display_name
                        .to_owned()
                        .unwrap_or(format!("Fan #{}", acc)),
                    thresholds: f.temperature_thresholds.to_owned(),
                })
            })
            .collect();

        self.critical_temperature = c.critical_temperature as u8;
        self.poll_interval = Duration::from_millis(c.ec_poll_interval);

        self.fan_configs
            .iter_mut()
            .for_each(|c| c.thresholds.sort());

        self.current_thr_indices = self.fan_configs.iter().map(|_| 0).collect();

        self.writer
            .refresh_config(
                c.read_write_words,
                c.register_write_configurations,
                &c.fan_configurations,
            )
            .context(Writer {})?;

        self.reader
            .refresh_config(c.read_write_words, &c.fan_configurations);

        Ok(())
    }

    //TODO: Refactoring
    /// Refresh the index of the current fan threshold according to the temperature (if necessary).
    /// Returns false if the threshold didn't need change.
    pub fn refresh_fan_threshold(&mut self, current_temp: f64, fan_index: usize) -> bool {
        let thresholds = &self.fan_configs[fan_index].thresholds;
        let current_thr_i = &mut self.current_thr_indices[fan_index];

        if current_temp as u8 >= thresholds[*current_thr_i].down_threshold
            && current_temp as u8 <= thresholds[*current_thr_i].up_threshold
        {
            return false;
        } else if current_temp as u8 <= thresholds[1].down_threshold {
            *current_thr_i = 0;
        } else if current_temp as u8 >= thresholds[thresholds.len() - 1].up_threshold {
            *current_thr_i = thresholds.len() - 1;
        } else if let Ok(i) = thresholds.binary_search_by(|el| match el {
            tmp if tmp.down_threshold > current_temp as u8 => Ordering::Greater,
            tmp if tmp.up_threshold < current_temp as u8 => Ordering::Less,
            _ => Ordering::Equal,
        }) {
            *current_thr_i = i;
        }

        true
    }

    /// Write the speed percent to the EC for the fan specified by `fan_index`.
    pub fn write_fan_speed(&mut self, fan_index: usize, speed_percent: f64) -> Result<()> {
        self.writer
            .write_speed_percent(fan_index, speed_percent)
            .context(Writer {})
    }

    /// Read the speed percent from the EC of the fan specified by `fan_index`.
    pub fn read_fan_speed(&mut self, fan_index: usize) -> Result<f64> {
        self.reader.read_speed_percent(fan_index).context(Reader {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::io::{Cursor, Read};

    static CONFIGS_PARSED: Lazy<Vec<FanControlConfigV2>> = Lazy::new(|| {
        std::fs::read_dir("nbfc_configs/Configs")
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| std::fs::File::open(e.path()).unwrap())
            .map(|mut e| {
                let mut buf = String::new();
                e.read_to_string(&mut buf).unwrap();
                buf
            })
            .map(|e| quick_xml::de::from_str::<FanControlConfigV2>(&e).unwrap())
            .collect()
    });
    #[test]
    fn refresh_config() {
        use is_sorted::IsSorted;
        CONFIGS_PARSED.iter().for_each(|c| {
            let ec = Cursor::new(vec![0u8; 256]);

            let mut task_struct: ECManager<_> = ECManager::new(ec);

            task_struct.refresh_control_config(c.clone()).unwrap();

            assert!(task_struct.critical_temperature == c.critical_temperature as u8);
            assert!(task_struct
                .fan_configs
                .iter_mut()
                .all(|f| { IsSorted::is_sorted(&mut f.thresholds.iter()) }));
            assert!(task_struct.current_thr_indices.iter().all(|e| *e == 0));
        });
    }

    #[test]
    fn select_threshold() {
        CONFIGS_PARSED.iter().for_each(|c| {
            let ec = Cursor::new(vec![0u8; 256]);

            let mut task_struct: ECManager<_> = ECManager::new(ec);

            task_struct.refresh_control_config(c.clone()).unwrap();

            for i in 0..c.fan_configurations.len() {
                let thresholds = &c.fan_configurations[i].temperature_thresholds;

                let very_high_temperature = 90.0;
                task_struct.refresh_fan_threshold(very_high_temperature, i);
                assert!(task_struct.current_thr_indices[i] == thresholds.len() - 1);

                let very_low_temperature = 20.0;
                task_struct.refresh_fan_threshold(very_low_temperature, i);
                assert!(
                    task_struct.current_thr_indices[i] == 0
                        || task_struct.current_thr_indices[i] == 1
                );

                // let mut rng = rand::thread_rng();

                // for t in 50..80 {
                //     println!("t°:{}", t);
                //     // let random_temp = rng.gen_range(40.0, 80.0);

                //     task_struct.refresh_fan_threshold(t as f64, i);

                //     let thr = task_struct.current_thr_indices[i];
                //     println!("thr:{}", thr);
                //     let excepted_thr = match task_struct.fan_configurations[i]
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
        });
    }

    // #[test]
    // fn requests() {

    // }
}
