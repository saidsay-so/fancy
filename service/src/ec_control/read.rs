/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use log::debug;

use std::io::{Error, Read, Seek, SeekFrom};

use super::RcWrapper;
use crate::nbfc::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// This strcture contains information for reading from the EC for a fan.
struct FanReadConfig {
    read_register: u8,
    max_speed_read: u16,
    min_speed_read: u16,
    read_percent_overrides: Option<Vec<FanSpeedPercentageOverride>>,
}

#[derive(Debug)]
/// A structure to manage reads from the EC.
pub(crate) struct ECReader<R: Read + Seek> {
    read_words: bool,
    ec_dev: RcWrapper<R>,
    fans_read_config: Vec<FanReadConfig>,
}

impl<R: Read + Seek> ECReader<R> {
    /// Initialize a reader.
    pub fn new(ec_dev: RcWrapper<R>) -> Self {
        ECReader {
            read_words: false,
            ec_dev,
            fans_read_config: Vec::new(),
        }
    }

    /// Refresh the configuration used for reading. NOTE: It doesn't read anything from the controller.
    pub fn refresh_config(&mut self, read_words: bool, fan_configs: &[FanConfiguration]) {
        self.read_words = read_words;
        self.fans_read_config = fan_configs
            .iter()
            .map(|fan| FanReadConfig {
                read_register: fan.read_register,
                min_speed_read: if fan.independent_read_min_max_values {
                    fan.min_speed_value_read
                } else {
                    fan.min_speed_value
                },
                max_speed_read: if fan.independent_read_min_max_values {
                    fan.max_speed_value_read
                } else {
                    fan.max_speed_value
                },
                read_percent_overrides: fan.fan_speed_percentage_overrides.as_ref().map(|f| {
                    f.iter()
                        .filter(|e| {
                            e.target_operation == Some(OverrideTargetOperation::Read)
                                || e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                        })
                        .map(|e| e.to_owned())
                        .collect::<Vec<FanSpeedPercentageOverride>>()
                }),
            })
            .collect();
    }

    /// Read the speed value for the fan specified at `fan_index`.
    //TODO: Check if the value is not out of the bounds
    pub fn read_speed_percent(&self, fan_index: usize) -> Result<f64> {
        let fan = &self.fans_read_config[fan_index];
        let read_off = SeekFrom::Start(fan.read_register as u64);
        let speed = self.read_value(read_off)?;

        let percentage: f64 = if let Some(speed_percent) =
            fan.read_percent_overrides.as_ref().and_then(|f| {
                f.iter()
                    .filter(|e| e.fan_speed_value == speed)
                    .map(|e| e.fan_speed_percentage)
                    .next()
            }) {
            speed_percent.into()
        } else {
            ((speed as f64 - fan.min_speed_read as f64)
                / (fan.max_speed_read as f64 - fan.min_speed_read as f64))
                * 100.0
        };

        Ok(percentage)
    }

    /// Low-level read function.
    // XXX: The function returns an u16 even if just a u8 is needed
    fn read_value(&self, read_off: SeekFrom) -> Result<u16> {
        // XXX: The buffer takes 2 bytes even if just one is needed
        let mut buf = [0u8; 2];
        let mut dev = (*self.ec_dev).borrow_mut();

        dev.seek(read_off)?;
        dev.read_exact(if self.read_words {
            &mut buf[..]
        } else {
            &mut buf[..=0]
        })?;

        debug!("Reading at offset {:?} the value {:?}", read_off, &buf);

        if self.read_words {
            Ok(u16::from_le_bytes(buf))
        } else {
            Ok(buf[0].into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use rand::Rng;
    use std::cell::RefCell;
    use std::io::{Cursor, Write};
    use std::rc::Rc;

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
            .map(|e| {
                quick_xml::de::from_str::<XmlFanControlConfigV2>(&e)
                    .unwrap()
                    .into()
            })
            .collect()
    });

    #[test]
    fn refresh() {
        CONFIGS_PARSED.iter().for_each(|c| {
            let ec = Cursor::new(vec![0; 256]);
            let ec = Rc::new(RefCell::new(ec));
            let mut reader = ECReader::new(Rc::clone(&ec));
            reader.refresh_config(c.read_write_words, &c.fan_configurations);

            assert_eq!(reader.read_words, c.read_write_words);

            assert_eq!(reader.fans_read_config.len(), c.fan_configurations.len());

            let mut i = 0;
            reader.fans_read_config.iter().for_each(|f| {
                assert_eq!(f.read_register, c.fan_configurations[i].read_register);
                let fan = &c.fan_configurations[i];
                let excepted_min = if fan.independent_read_min_max_values {
                    fan.min_speed_value_read
                } else {
                    fan.min_speed_value
                };
                assert_eq!(f.min_speed_read, excepted_min);

                let excepted_max = if fan.independent_read_min_max_values {
                    fan.max_speed_value_read
                } else {
                    fan.max_speed_value
                };
                assert_eq!(f.max_speed_read, excepted_max);

                if let Some(ref overrides) = f.read_percent_overrides {
                    let excepted_overrides = c.fan_configurations[i]
                        .fan_speed_percentage_overrides
                        .as_ref()
                        .unwrap();
                    assert_eq!(
                        overrides.len(),
                        excepted_overrides
                            .iter()
                            .filter(|o| o.target_operation
                                == Some(OverrideTargetOperation::ReadWrite)
                                || o.target_operation == Some(OverrideTargetOperation::Read))
                            .count()
                    );

                    overrides.iter().for_each(|o| {
                        assert!(excepted_overrides.iter().any(|e| e == o));
                    });
                }
                i += 1;
            });
        });
    }

    #[test]
    fn read_register_value() {
        CONFIGS_PARSED.iter().for_each(|c| {
            let mut rng = rand::thread_rng();
            let ec = Cursor::new(vec![0; 256]);
            let ec = Rc::new(RefCell::new(ec));
            let mut reader = ECReader::new(Rc::clone(&ec));
            reader.refresh_config(c.read_write_words, &c.fan_configurations);
            let mut i = 0;

            for fan in &c.fan_configurations {
                let min_speed_read = if fan.independent_read_min_max_values {
                    fan.min_speed_value_read
                } else {
                    fan.min_speed_value
                };

                let max_speed_read = if fan.independent_read_min_max_values {
                    fan.max_speed_value_read
                } else {
                    fan.max_speed_value
                };

                let random_value: u16 = rng.gen_range(
                    std::cmp::min(min_speed_read, max_speed_read)
                        ..std::cmp::max(min_speed_read, max_speed_read),
                );

                for write_value in [min_speed_read, max_speed_read, random_value].iter() {
                    let excepted_value = if let Some(v) =
                        fan.fan_speed_percentage_overrides.as_ref().and_then(|e| {
                            e.iter()
                                .filter(|e| {
                                    e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                        || e.target_operation == Some(OverrideTargetOperation::Read)
                                })
                                .filter(|e| e.fan_speed_value == *write_value)
                                .next()
                        }) {
                        v.fan_speed_percentage as f64
                    } else {
                        ((*write_value as f64 - min_speed_read as f64)
                            / (max_speed_read as f64 - min_speed_read as f64))
                            * 100.0
                    };

                    write(
                        Rc::clone(&ec),
                        fan.read_register.into(),
                        &write_value.to_le_bytes(),
                    );

                    let value = reader.read_speed_percent(i).unwrap();

                    assert_eq!(excepted_value, value);
                }

                i += 1;
            }
        });
    }

    #[test]
    fn read_overrides() {
        CONFIGS_PARSED.iter().for_each(|c| {
            let ec = Cursor::new(vec![0; 256]);
            let ec = Rc::new(RefCell::new(ec));
            let mut reader = ECReader::new(Rc::clone(&ec));
            reader.refresh_config(c.read_write_words, &c.fan_configurations);
            let mut i = 0;

            for fan in &c.fan_configurations {
                if let Some(fan_override) = fan.fan_speed_percentage_overrides.as_ref() {
                    for override_s in fan_override.iter().filter(|e| {
                        e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                            || e.target_operation == Some(OverrideTargetOperation::Read)
                    }) {
                        write(
                            Rc::clone(&ec),
                            fan.read_register as u64,
                            &override_s.fan_speed_value.to_le_bytes(),
                        );
                        let value = reader.read_speed_percent(i).unwrap();
                        let excepted_value = override_s.fan_speed_percentage as f64;

                        assert_eq!(excepted_value, value);
                    }
                }
                i += 1;
            }
        });
    }

    fn write(ec: RcWrapper<Cursor<Vec<u8>>>, pos: u64, value: &[u8]) {
        let mut ec = (*ec).borrow_mut();
        ec.set_position(pos);
        ec.write(value).unwrap();
    }
}
