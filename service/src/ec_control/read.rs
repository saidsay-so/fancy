/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
pub use async_std::io::Error;
use async_std::io::SeekFrom;

use crate::ec_control::EcRead;
use crate::nbfc::*;

use super::ArcWrapper;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// This strcture contains information for reading from the EC for a fan.
struct FanReadConfig {
    read_register: u8,
    max_speed_read: u16,
    min_speed_read: u16,
    read_percent_overrides: Vec<FanSpeedPercentageOverride>,
}

#[derive(Debug)]
/// A structure to manage reads from the EC.
pub(crate) struct ECReader<R: EcRead> {
    read_words: bool,
    ec_dev: ArcWrapper<R>,
    fans_read_config: Vec<FanReadConfig>,
}

impl<R: EcRead> ECReader<R> {
    /// Initialize a reader.
    pub fn new(ec_dev: ArcWrapper<R>) -> Self {
        ECReader {
            read_words: false,
            ec_dev,
            fans_read_config: Vec::new(),
        }
    }

    /// Refresh the configuration used for reading.
    /// NOTE: It doesn't read anything from the controller.
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
                read_percent_overrides: fan.fan_speed_percentage_overrides.as_ref().map_or_else(
                    Vec::new,
                    |f| {
                        f.iter()
                            .filter(|e| {
                                e.target_operation == Some(OverrideTargetOperation::Read)
                                    || e.target_operation
                                        == Some(OverrideTargetOperation::ReadWrite)
                            })
                            .map(|e| e.to_owned())
                            .collect::<Vec<FanSpeedPercentageOverride>>()
                    },
                ),
            })
            .collect();
    }

    /// Read the speed value for the fan specified at `fan_index`.
    pub async fn read_speed_percent(&self, fan_index: usize) -> Result<f64> {
        let fan = &self.fans_read_config[fan_index];
        let read_off = SeekFrom::Start(fan.read_register as u64);
        let speed = self.read_value(read_off).await?;

        let percentage: f64 = if let Some(speed_percent) = fan
            .read_percent_overrides
            .iter()
            .find(|e| e.fan_speed_value == speed)
            .map(|e| e.fan_speed_percentage)
        {
            speed_percent.into()
        } else {
            ((speed as f64 - fan.min_speed_read as f64)
                / (fan.max_speed_read as f64 - fan.min_speed_read as f64))
                * 100.0
        };

        Ok(percentage.clamp(0.0, 100.0))
    }

    /// Low-level read function.
    async fn read_value(&self, read_off: SeekFrom) -> Result<u16> {
        let mut buf = [0u8; 2];
        let mut dev = self.ec_dev.lock().await;

        dev.read_bytes(
            read_off,
            if self.read_words {
                &mut buf[..]
            } else {
                &mut buf[..=0]
            },
        )
        .await?;

        if self.read_words {
            Ok(u16::from_le_bytes(buf))
        } else {
            Ok(buf[0].into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::parsed_configs;
    use async_std::io::prelude::*;
    use async_std::io::Cursor;
    use async_std::sync::Arc;
    use async_std::sync::Mutex;
    use rand::Rng;
    use rayon::prelude::*;
    use rstest::rstest;
    use smol::block_on;

    use super::*;

    #[rstest]
    fn refresh(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut reader = ECReader::new(Arc::clone(&ec));
                reader.refresh_config(c.read_write_words, &c.fan_configurations);

                assert_eq!(reader.read_words, c.read_write_words);

                assert_eq!(reader.fans_read_config.len(), c.fan_configurations.len());

                for (f, expected_f) in reader
                    .fans_read_config
                    .iter()
                    .zip(c.fan_configurations.iter())
                {
                    assert_eq!(f.read_register, expected_f.read_register);
                    let fan = &expected_f;
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

                    let excepted_overrides = expected_f
                        .fan_speed_percentage_overrides
                        .as_ref()
                        .map_or(Vec::new(), |o| {
                            o.iter()
                                .cloned()
                                .filter(|o| {
                                    o.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                        || o.target_operation == Some(OverrideTargetOperation::Read)
                                })
                                .collect::<Vec<_>>()
                        });
                    assert_eq!(f.read_percent_overrides, excepted_overrides);
                }
            })
        });
    }

    #[rstest]
    fn read_register_value(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let mut rng = rand::thread_rng();
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut reader = ECReader::new(Arc::clone(&ec));
                reader.refresh_config(c.read_write_words, &c.fan_configurations);

                for (i, fan) in c.fan_configurations.iter().enumerate() {
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

                    for write_value in [min_speed_read, random_value, max_speed_read].iter() {
                        let excepted_value = if let Some(v) =
                            fan.fan_speed_percentage_overrides.as_ref().and_then(|e| {
                                e.iter().find(|e| {
                                    (e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                        || e.target_operation
                                            == Some(OverrideTargetOperation::Read))
                                        && e.fan_speed_value == *write_value
                                })
                            }) {
                            v.fan_speed_percentage as f64
                        } else {
                            ((*write_value as f64 - min_speed_read as f64)
                                / (max_speed_read as f64 - min_speed_read as f64))
                                * 100.0
                        };

                        write(
                            Arc::clone(&ec),
                            fan.read_register.into(),
                            &write_value.to_le_bytes(),
                        )
                        .await;

                        let value = reader.read_speed_percent(i).await.unwrap();

                        assert_eq!(excepted_value, value);
                    }
                }
            })
        });
    }

    #[rstest]
    fn read_overrides(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut reader = ECReader::new(Arc::clone(&ec));
                reader.refresh_config(c.read_write_words, &c.fan_configurations);

                for (i, fan) in c.fan_configurations.iter().enumerate() {
                    if let Some(fan_override) = fan.fan_speed_percentage_overrides.as_ref() {
                        for override_s in fan_override.iter().filter(|e| {
                            e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                || e.target_operation == Some(OverrideTargetOperation::Read)
                        }) {
                            write(
                                Arc::clone(&ec),
                                fan.read_register as u64,
                                &override_s.fan_speed_value.to_le_bytes(),
                            )
                            .await;

                            let value = reader.read_speed_percent(i).await.unwrap();
                            let excepted_value = override_s.fan_speed_percentage as f64;

                            assert_eq!(excepted_value, value);
                        }
                    }
                }
            })
        });
    }

    async fn write(ec: ArcWrapper<Cursor<Vec<u8>>>, pos: u64, value: &[u8]) {
        let ec = &mut *ec.lock().await;
        ec.set_position(pos);
        ec.write(value).await.unwrap();
    }
}
