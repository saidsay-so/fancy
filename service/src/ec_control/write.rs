/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use async_std::io::{Error, SeekFrom};

use crate::ec_control::EcWrite;
use crate::nbfc::*;

use super::ArcWrapper;

#[derive(Debug)]
/// Contains information about writing to the EC for a fan.
struct FanWriteConfig {
    write_register: u8,
    reset_required: bool,
    reset_value: Option<u16>,
    max_speed: u16,
    min_speed: u16,
    write_percent_overrides: Vec<FanSpeedPercentageOverride>,
}

#[derive(Debug)]
/// Manages writes to the EC.
pub(crate) struct ECWriter<W: EcWrite> {
    on_write_reg_confs: Vec<RegisterWriteConfiguration>,
    init_reg_confs: Vec<RegisterWriteConfiguration>,
    fans_write_config: Vec<FanWriteConfig>,
    write_words: bool,
    ec_dev: ArcWrapper<W>,
}

type Result<T = ()> = std::result::Result<T, Error>;

impl<W: EcWrite> ECWriter<W> {
    /// Initialize a new writer.
    pub fn new(ec_dev: ArcWrapper<W>) -> Self {
        ECWriter {
            on_write_reg_confs: Vec::new(),
            init_reg_confs: Vec::new(),
            fans_write_config: Vec::new(),
            write_words: false,
            ec_dev,
        }
    }

    /// Refresh the configuration used for the writer.
    /// NOTE: This function does write the required values to initialize the controller (using `init_write`).
    pub async fn refresh_config(
        &mut self,
        write_words: bool,
        reg_confs: Option<Vec<RegisterWriteConfiguration>>,
        fan_configs: &[FanConfiguration],
    ) -> Result {
        if let Some(reg_confs) = reg_confs {
            self.on_write_reg_confs = reg_confs
                .iter()
                .filter(|r| r.write_occasion == Some(RegisterWriteOccasion::OnWriteFanSpeed))
                .cloned()
                .collect();

            self.init_reg_confs = reg_confs
                .iter()
                .filter(|r| r.write_occasion == Some(RegisterWriteOccasion::OnInitialization))
                .cloned()
                .collect();
        }

        self.write_words = write_words;

        self.fans_write_config = fan_configs
            .iter()
            .map(|fan| FanWriteConfig {
                write_register: fan.write_register,
                reset_required: fan.reset_required,
                reset_value: fan.fan_speed_reset_value,
                min_speed: fan.min_speed_value,
                max_speed: fan.max_speed_value,
                write_percent_overrides: fan.fan_speed_percentage_overrides.as_ref().map_or_else(
                    Vec::new,
                    |f| {
                        f.iter()
                            .filter(|e| {
                                e.target_operation == Some(OverrideTargetOperation::Write)
                                    || e.target_operation
                                        == Some(OverrideTargetOperation::ReadWrite)
                            })
                            .cloned()
                            .collect()
                    },
                ),
            })
            .collect();

        self.init_write().await
    }

    /// Function to call before starting to write. It initialize the EC controller so it can be used.
    async fn init_write(&self) -> Result {
        for reg_conf in self.init_reg_confs.iter() {
            let write_off = SeekFrom::Start(reg_conf.register as u64);
            self.write_value(false, write_off, &reg_conf.value.to_le_bytes())
                .await?;
        }

        for c in &self.fans_write_config {
            if let Some(value) = c.reset_value {
                let write_off = SeekFrom::Start(c.write_register as u64);
                self.write_value(self.write_words, write_off, &value.to_le_bytes())
                    .await?;
            }
        }

        Ok(())
    }

    /// Reset the EC. Resets all the registers (even when it's not required) if `reset_all` is true.
    pub async fn reset(&self, reset_all: bool) -> Result {
        for reg_conf in self.init_reg_confs.iter() {
            if reset_all || reg_conf.reset_required {
                let write_off = SeekFrom::Start(reg_conf.register as u64);
                if let Some(value) = reg_conf.reset_value {
                    self.write_value(false, write_off, &value.to_le_bytes())
                        .await?;
                }
            }
        }

        for reg_conf in self.on_write_reg_confs.iter() {
            if reset_all || reg_conf.reset_required {
                let write_off = SeekFrom::Start(reg_conf.register as u64);
                if let Some(value) = reg_conf.reset_value {
                    self.write_value(false, write_off, &value.to_le_bytes())
                        .await?;
                }
            }
        }

        for c in &self.fans_write_config {
            if reset_all || c.reset_required {
                if let Some(value) = c.reset_value {
                    let write_off = SeekFrom::Start(c.write_register as u64);
                    self.write_value(self.write_words, write_off, &value.to_le_bytes())
                        .await?;
                }
            }
        }

        Ok(())
    }

    /// Write the `speed_percent` to the EC for the fan specified by `fan_index`.
    pub async fn write_speed_percent(&self, fan_index: usize, speed_percent: f64) -> Result {
        for reg_conf in self.on_write_reg_confs.iter() {
            let write_off = SeekFrom::Start(reg_conf.register as u64);
            self.write_value(false, write_off, &reg_conf.value.to_le_bytes())
                .await?;
        }

        let fan = &self.fans_write_config[fan_index];
        let speed = if let Some(speed_value) = fan
            .write_percent_overrides
            .iter()
            .find(|e| (e.fan_speed_percentage as f64 - speed_percent).abs() < f64::EPSILON)
            .map(|e| e.fan_speed_value)
        {
            speed_value.to_le_bytes()
        } else {
            ((fan.min_speed as f64
                + (((fan.max_speed as f64 - fan.min_speed as f64) * speed_percent) / 100.0))
                .round() as u16)
                .to_le_bytes()
        };

        let write_off = SeekFrom::Start(fan.write_register as u64);
        self.write_value(self.write_words, write_off, &speed).await
    }

    /// Low-level write function.
    async fn write_value(&self, write_word: bool, write_off: SeekFrom, value: &[u8]) -> Result {
        let mut dev = self.ec_dev.lock().await;

        dev.write_bytes(write_off, if write_word { &value } else { &value[..=0] })
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::parsed_configs;
    use async_std::io::Cursor;
    use async_std::sync::{Arc, Mutex};
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
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                assert_eq!(
                    writer.on_write_reg_confs,
                    c.register_write_configurations
                        .as_ref()
                        .map_or(Vec::new(), |c| c
                            .iter()
                            .cloned()
                            .filter(|c| c.write_occasion
                                == Some(RegisterWriteOccasion::OnWriteFanSpeed))
                            .collect::<Vec<_>>())
                );

                assert_eq!(
                    writer.init_reg_confs,
                    c.register_write_configurations
                        .as_ref()
                        .map_or(Vec::new(), |c| c
                            .iter()
                            .cloned()
                            .filter(|c| c.write_occasion
                                == Some(RegisterWriteOccasion::OnInitialization))
                            .collect::<Vec<_>>())
                );

                assert_eq!(writer.fans_write_config.len(), c.fan_configurations.len());

                for (f, expected_f) in writer
                    .fans_write_config
                    .iter()
                    .zip(c.fan_configurations.iter())
                {
                    assert_eq!(f.reset_required, expected_f.reset_required);
                    assert_eq!(f.write_register, expected_f.write_register);
                    assert_eq!(f.reset_value, expected_f.fan_speed_reset_value);
                    assert_eq!(f.min_speed, expected_f.min_speed_value);
                    assert_eq!(f.max_speed, expected_f.max_speed_value);

                    let excepted_overrides = expected_f
                        .fan_speed_percentage_overrides
                        .as_ref()
                        .map_or(Vec::new(), |o| {
                            o.iter()
                                .cloned()
                                .filter(|o| {
                                    o.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                        || o.target_operation
                                            == Some(OverrideTargetOperation::Write)
                                })
                                .collect::<Vec<_>>()
                        });

                    assert_eq!(f.write_percent_overrides.clone(), excepted_overrides);
                }

                assert_eq!(writer.write_words, c.read_write_words);
            })
        });
    }

    #[rstest]
    fn reset_only_required(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                {
                    let ec_lock = &mut *ec.lock().await;
                    let buffer = ec_lock.get_mut();
                    let _ = std::mem::replace(buffer, vec![0; 256]);
                }

                writer.reset(false).await.unwrap();

                let ec_lock = &mut *ec.lock().await;
                let buffer = ec_lock.get_ref();

                if let Some(reg_confs) = c.register_write_configurations.as_ref() {
                    for reg_conf in reg_confs.iter() {
                        let write_off = reg_conf.register as usize;
                        let excepted_value = if reg_conf.reset_required {
                            reg_conf.reset_value.unwrap()
                        } else {
                            0
                        };

                        let value = buffer[write_off];
                        assert_eq!(excepted_value, value);
                    }
                }

                for fan in &c.fan_configurations {
                    if fan.reset_required {
                        let write_off = fan.write_register as usize;
                        let excepted_value = &fan.fan_speed_reset_value.unwrap().to_le_bytes();
                        let value = if c.read_write_words {
                            &buffer[write_off..=write_off + 1]
                        } else {
                            &buffer[write_off..=write_off]
                        };
                        assert_eq!(
                            if c.read_write_words {
                                &excepted_value[..]
                            } else {
                                &excepted_value[..=0]
                            },
                            value
                        );
                    }
                }
            })
        });
    }

    #[rstest]
    fn reset_all(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                {
                    let ec_lock = &mut *ec.lock().await;
                    let buffer = ec_lock.get_mut();
                    let _ = std::mem::replace(buffer, vec![0; 256]);
                }

                writer.reset(true).await.unwrap();

                let ec_lock = &mut *ec.lock().await;
                let buffer = ec_lock.get_ref();

                if let Some(reg_confs) = c.register_write_configurations.as_ref() {
                    for reg_conf in reg_confs.iter().filter(|e| e.reset_value.is_some()) {
                        let write_off = reg_conf.register as usize;
                        let excepted_value = reg_conf.reset_value.unwrap();

                        let value = buffer[write_off];
                        assert_eq!(excepted_value, value);
                    }

                    for fan in &c.fan_configurations {
                        if fan.reset_required {
                            let write_off = fan.write_register as usize;
                            let excepted_value = &fan.fan_speed_reset_value.unwrap().to_le_bytes();

                            let value = if c.read_write_words {
                                &buffer[write_off..=write_off + 1]
                            } else {
                                &buffer[write_off..=write_off]
                            };
                            assert_eq!(
                                if c.read_write_words {
                                    &excepted_value[..]
                                } else {
                                    &excepted_value[..=0]
                                },
                                value
                            );
                        }
                    }
                }
            })
        });
    }

    #[rstest]
    fn init_write(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                let ec_lock = &*ec.lock().await;
                let buffer = ec_lock.get_ref();

                if let Some(reg_confs) = c.register_write_configurations.as_ref() {
                    for reg_conf in reg_confs.iter().filter(|e| {
                        e.write_occasion == Some(RegisterWriteOccasion::OnInitialization)
                    }) {
                        let write_off = reg_conf.register as usize;
                        let excepted_value = reg_conf.value;

                        let value = buffer[write_off];
                        assert_eq!(excepted_value, value);
                    }

                    for fan in &c.fan_configurations {
                        if fan.reset_required {
                            let write_off = fan.write_register as usize;
                            let excepted_value = &fan.fan_speed_reset_value.unwrap().to_le_bytes();

                            let value = if c.read_write_words {
                                &buffer[write_off..=write_off + 1]
                            } else {
                                &buffer[write_off..=write_off]
                            };
                            assert_eq!(
                                if c.read_write_words {
                                    &excepted_value[..]
                                } else {
                                    &excepted_value[..=0]
                                },
                                value
                            );
                        }
                    }
                }
            })
        });
    }

    #[rstest]
    fn write_overrides(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                for (fan, i) in c.fan_configurations.iter().zip(0..) {
                    if let Some(fan_override) = fan.fan_speed_percentage_overrides.as_ref() {
                        for override_s in fan_override.iter().filter(|e| {
                            e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                || e.target_operation == Some(OverrideTargetOperation::Write)
                        }) {
                            writer
                                .write_speed_percent(i, override_s.fan_speed_percentage.into())
                                .await
                                .unwrap();

                            let ec_lock = ec.lock().await;
                            let buffer = ec_lock.get_ref();

                            let write_off = fan.write_register as usize;
                            let buf = override_s.fan_speed_value.to_le_bytes();
                            let excepted_value = if c.read_write_words {
                                &buf[..]
                            } else {
                                &buf[..=0]
                            };

                            let value = {
                                if c.read_write_words {
                                    &buffer[write_off..=write_off + 1]
                                } else {
                                    &buffer[write_off..write_off + 1]
                                }
                            };

                            assert_eq!(excepted_value, value);
                        }
                    }
                }
            })
        });
    }

    #[rstest]
    fn on_write_confs(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                writer.write_speed_percent(0, 0.0).await.unwrap();

                let ec_lock = ec.lock().await;
                let buffer = ec_lock.get_ref();

                if let Some(reg_confs) = c.register_write_configurations.as_ref() {
                    for reg_conf in reg_confs.iter().filter(|e| {
                        e.write_occasion == Some(RegisterWriteOccasion::OnWriteFanSpeed)
                    }) {
                        let write_off = reg_conf.register as usize;
                        let excepted_value = reg_conf.value;

                        let value = buffer[write_off];
                        assert_eq!(excepted_value, value);
                    }
                }
            })
        });
    }

    #[rstest]
    fn write_good_offset(parsed_configs: &Vec<FanControlConfigV2>) {
        parsed_configs.par_iter().for_each(|c| {
            block_on(async {
                let ec = Cursor::new(vec![0; 256]);
                let ec = Arc::new(Mutex::new(ec));
                let mut writer = ECWriter::new(Arc::clone(&ec));
                writer
                    .refresh_config(
                        c.read_write_words,
                        c.register_write_configurations.clone(),
                        &c.fan_configurations,
                    )
                    .await
                    .unwrap();

                let speed_percent = 0.0;

                for (fan, i) in c.fan_configurations.iter().zip(0..) {
                    writer.write_speed_percent(i, speed_percent).await.unwrap();

                    let ec_lock = ec.lock().await;
                    let buffer = ec_lock.get_ref();

                    let write_off = fan.write_register as usize;
                    let value = {
                        if c.read_write_words {
                            let buf = &buffer[write_off..=write_off + 1];
                            u16::from_le_bytes(buf.try_into().unwrap())
                        } else {
                            buffer[write_off] as u16
                        }
                    };

                    let excepted_value = if let Some(v) =
                        fan.fan_speed_percentage_overrides.as_ref().and_then(|e| {
                            e.iter()
                                .filter(|e| {
                                    e.target_operation == Some(OverrideTargetOperation::ReadWrite)
                                        || e.target_operation
                                            == Some(OverrideTargetOperation::Write)
                                })
                                .filter(|e| e.fan_speed_percentage as f64 == speed_percent)
                                .next()
                        }) {
                        v.fan_speed_value
                    } else {
                        (fan.min_speed_value as f64
                            + (((fan.max_speed_value as f64 - fan.min_speed_value as f64)
                                * speed_percent)
                                / 100.0))
                            .round() as u16
                    };

                    assert_eq!(value, excepted_value);
                }
            })
        });
    }
}
