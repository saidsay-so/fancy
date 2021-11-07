/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use log::info;
use phf::phf_map;
use quick_xml::de::from_str as xml_from_str;
use serde_json::de::from_str as json_from_str;
use snafu::{ensure, ResultExt, Snafu};

use std::fs::{read_dir, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::nbfc::{
    check_control_config, CheckControlConfigError, FanControlConfigV2, XmlFanControlConfigV2,
};

#[derive(Debug, Snafu)]
pub(crate) enum ControlConfigLoadError {
    #[snafu(display(
        "Error occurred while trying to load control config `{}`: {}",
        name,
        source
    ))]
    Loading {
        name: String,
        source: std::io::Error,
    },

    #[snafu(display("Error while iterating through `{}`",dir.display()))]
    IterDir {
        dir: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "Error occurred while deserializing control config `{}`: {}",
        name,
        source
    ))]
    ControlJsonDeserialize {
        name: String,
        source: serde_json::Error,
    },

    #[snafu(display(
        "Error occurred while deserializing control config `{}`: {}",
        name,
        source
    ))]
    ControlXmlDeserialize {
        name: String,
        source: quick_xml::DeError,
    },

    #[snafu(display("The control config `{}` does not exist", name))]
    InexistentConfig { name: String },

    #[snafu(display("The control config name `{}` contains invalid characters", name))]
    InvalidChars { name: String },

    #[snafu(display("Error occured while checking control config `{}`: {}", name, source))]
    Check {
        name: String,
        source: CheckControlConfigError,
    },
}

const INVALID_CHARS: &[char] = &['.', '/'];

type Result<T> = std::result::Result<T, ControlConfigLoadError>;

type Deserializer = fn(&str, String) -> Result<FanControlConfigV2>;

const SUPPORTED_EXTENSIONS: phf::Map<&str, Deserializer> = phf_map! {
    "xml" => xml_deserializer,
    "json" => json_deserializer,
};

fn xml_deserializer(name: &str, buf: String) -> Result<FanControlConfigV2> {
    Ok(xml_from_str::<XmlFanControlConfigV2>(&buf)
        .context(ControlXmlDeserialize { name })?
        .into())
}

fn json_deserializer(name: &str, buf: String) -> Result<FanControlConfigV2> {
    Ok(json_from_str::<FanControlConfigV2>(&buf)
        .context(ControlJsonDeserialize { name })?
        .into())
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ControlConfigLoader {
    allowed_paths: Vec<PathBuf>,
    follow_dirs: bool,
}

impl ControlConfigLoader {
    pub(crate) fn new(follow_dirs: bool) -> Self {
        Self {
            allowed_paths: Vec::new(),
            follow_dirs,
        }
    }

    pub(crate) fn add_path(&mut self, p: &Path) -> Result<bool> {
        let p = p.to_owned();
        if !self.allowed_paths.contains(&p) && p.is_dir() {
            if self.follow_dirs {
                for entry in read_dir(&p).context(IterDir { dir: &p })? {
                    let entry = entry.context(IterDir { dir: &p })?.path();
                    if entry.is_dir() {
                        self.add_path(&entry)?;
                    }
                }
            }
            self.allowed_paths.push(p);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_file_path<S: AsRef<str>>(&self, name: S) -> Result<(PathBuf, Deserializer)> {
        let name = name.as_ref();
        ensure!(!name.contains(&INVALID_CHARS[..]), InvalidChars { name });

        let path = self.allowed_paths.iter().find_map(|path| {
            let mut path = path.join(name);

            if let Some(ext) = SUPPORTED_EXTENSIONS.keys().find(|ext| {
                path.set_extension(ext);
                path.is_file()
            }) {
                Some((path, SUPPORTED_EXTENSIONS[ext]))
            } else {
                None
            }
        });

        if let Some(path) = path {
            Ok(path)
        } else {
            Err(InexistentConfig { name }.build())
        }
    }

    /// Loads the fan control configuration.
    pub(crate) fn load_control_config<S: AsRef<str>>(&self, name: S) -> Result<FanControlConfigV2> {
        let name = name.as_ref();
        info!("Loading fan control configuration '{}'", name);

        let (path, de) = self.get_file_path(name)?;

        let mut config_file = File::open(path).context(Loading { name })?;

        let mut buf = String::new();
        config_file
            .read_to_string(&mut buf)
            .context(Loading { name })?;

        let c = de(name, buf)?;

        Ok(c)
    }

    /// Test if the fan control config provided can be loaded.
    pub(crate) fn test_control_config<S: AsRef<str>>(
        &self,
        name: S,
        check_config: bool,
    ) -> Result<()> {
        let name = name.as_ref();
        info!("Testing fan control configuration '{}'", name);

        let (path, de) = self.get_file_path(name)?;

        File::open(path)
            .context(Loading { name })
            .and_then(|mut f| {
                let mut buf = String::new();
                f.read_to_string(&mut buf).context(Loading { name })?;

                let c = de(name, buf)?;

                if !check_config {
                    return Ok(());
                }

                check_control_config(&c).context(Check { name })
            })
            .map(|_| {})
    }
}

#[cfg(test)]
mod tests {
    use crate::nbfc::*;
    use std::fs::read_to_string;

    use super::*;
    use rstest::*;

    #[fixture]
    fn not_follow_loader() -> ControlConfigLoader {
        let mut loader = ControlConfigLoader::new(false);
        loader.add_path(&PathBuf::from("tests")).unwrap();
        loader
    }

    #[fixture]
    fn follow_loader() -> ControlConfigLoader {
        let mut loader = ControlConfigLoader::new(true);
        loader.add_path(&PathBuf::from("tests")).unwrap();
        loader
    }

    #[rstest]
    fn add_new_path(mut not_follow_loader: ControlConfigLoader) {
        assert!(not_follow_loader.allowed_paths.len() == 1);

        assert!(not_follow_loader
            .add_path(&PathBuf::from("tests/not_follow"))
            .unwrap());

        assert!(!not_follow_loader
            .add_path(&PathBuf::from("tests/not_follow"))
            .unwrap());
    }

    #[rstest]
    fn follow_dirs(follow_loader: ControlConfigLoader) {
        const LAYOUT: &[&str] = &[
            "tests",
            "tests/follow",
            "tests/follow/json",
            "tests/not_follow",
        ];

        assert!(LAYOUT
            .iter()
            .all(|dir| follow_loader.allowed_paths.contains(&PathBuf::from(dir))));
    }

    #[rstest]
    fn get_nested_path(
        mut not_follow_loader: ControlConfigLoader,
        follow_loader: ControlConfigLoader,
    ) {
        assert!(not_follow_loader
            .add_path(&PathBuf::from("tests/follow"))
            .unwrap());
        assert!(not_follow_loader.get_file_path("valid_json").is_err());

        assert!(follow_loader.get_file_path("valid_json").is_ok());

        let (path, deserializer) = follow_loader.get_file_path("valid_json").unwrap();
        assert_eq!(path, PathBuf::from("tests/follow/json/valid_json.json"));

        let excepted_config = FanControlConfigV2 {
            notebook_model: Some("HP Envy X360 13-ag0xxx Ryzen-APU".to_string()),
            author: Some("Daniel Andersen".to_string()),
            ec_poll_interval: 1000,
            read_write_words: true,
            critical_temperature: 90,
            fan_configurations: [FanConfiguration {
                read_register: 149,
                write_register: 148,
                min_speed_value: 175,
                max_speed_value: 70,
                independent_read_min_max_values: false,
                min_speed_value_read: 0,
                max_speed_value_read: 0,
                reset_required: false,
                fan_speed_reset_value: Some(255),
                fan_display_name: Some("CPU fan".to_string()),
                temperature_thresholds: [
                    TemperatureThreshold {
                        up_threshold: 0,
                        down_threshold: 0,
                        fan_speed: 0.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 60,
                        down_threshold: 48,
                        fan_speed: 10.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 63,
                        down_threshold: 55,
                        fan_speed: 20.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 66,
                        down_threshold: 59,
                        fan_speed: 50.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 68,
                        down_threshold: 63,
                        fan_speed: 70.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 71,
                        down_threshold: 67,
                        fan_speed: 100.0,
                    },
                ]
                .to_vec(),
                fan_speed_percentage_overrides: Some(
                    [FanSpeedPercentageOverride {
                        fan_speed_percentage: 0.0,
                        fan_speed_value: 255,
                        target_operation: Some(OverrideTargetOperation::ReadWrite),
                    }]
                    .to_vec(),
                ),
            }]
            .to_vec(),
            register_write_configurations: Some(
                [RegisterWriteConfiguration {
                    write_mode: RegisterWriteMode::Set,
                    write_occasion: Some(RegisterWriteOccasion::OnInitialization),
                    register: 147,
                    value: 20,
                    reset_required: true,
                    reset_value: Some(4),
                    reset_write_mode: None,
                    description: Some("Set EC to manual control".to_string()),
                }]
                .to_vec(),
            ),
        };

        assert_eq!(
            deserializer("valid_json", read_to_string(&path).unwrap()).unwrap(),
            excepted_config
        );

        assert_eq!(
            follow_loader.load_control_config("valid_json").unwrap(),
            excepted_config
        );

        let (path, deserializer) = follow_loader.get_file_path("valid_xml").unwrap();
        assert_eq!(path, PathBuf::from("tests/follow/xml/valid_xml.xml"));

        assert_eq!(
            deserializer("valid_xml", read_to_string(&path).unwrap()).unwrap(),
            excepted_config
        );

        assert_eq!(
            follow_loader.load_control_config("valid_xml").unwrap(),
            excepted_config
        );
    }

    #[rstest]
    fn test_config(follow_loader: ControlConfigLoader) {
        assert!(follow_loader
            .test_control_config("valid_json", false)
            .is_ok());
        assert!(follow_loader
            .test_control_config("valid_json", true)
            .is_ok());

        assert!(follow_loader
            .test_control_config("valid_xml", false)
            .is_ok());
        assert!(follow_loader.test_control_config("valid_xml", true).is_ok());

        assert!(follow_loader.test_control_config("invalid", false).is_err());
        assert!(follow_loader.test_control_config("invalid", true).is_err());

        assert!(follow_loader
            .test_control_config("not_complete_config", false)
            .is_ok());
        assert!(follow_loader
            .test_control_config("not_complete_config", true)
            .is_err());
    }
}
