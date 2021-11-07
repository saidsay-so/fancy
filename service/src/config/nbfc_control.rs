/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use log::info;
use phf::phf_map;
use quick_xml::de::from_str as xml_from_str;
use serde_json::de::from_str as json_from_str;
use snafu::{ensure, ResultExt, Snafu};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

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
}

impl ControlConfigLoader {
    pub(crate) fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    pub(crate) fn add_path(&mut self, p: PathBuf) -> bool {
        if p.is_dir() {
            self.allowed_paths.push(p);
            true
        } else {
            false
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
