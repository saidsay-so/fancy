use log::info;
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use once_cell::sync::Lazy;
use quick_xml::de::from_str as xml_from_str;
use snafu::{ensure, ResultExt, Snafu};

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::constants::ROOT_CONFIG_PATH;
use crate::nbfc::{
    check_control_config, CheckControlConfigError, FanControlConfigV2, XmlFanControlConfigV2,
};

static CONTROL_CONFIGS_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| ROOT_CONFIG_PATH.join("configs"));
#[derive(Debug, Snafu)]
pub(crate) enum ControlConfigLoadError {
    #[snafu(display(
        "Error occured while trying to load control config `{}`: {}",
        name,
        source
    ))]
    Loading {
        name: String,
        source: std::io::Error,
    },

    #[snafu(display(
        "Error occured while deserializing control config `{}`: {}",
        name,
        source
    ))]
    ControlXmlDeserialize {
        name: String,
        source: quick_xml::DeError,
    },

    #[snafu(display("The config {} contains invalid characters", name))]
    InvalidChars { name: String },

    #[snafu(display("Error occured while checking control config `{}`: {}", name, source))]
    Check {
        name: String,
        source: CheckControlConfigError,
    },
}
const INVALID_CHARS: [char; 2] = ['.', '/'];

type Result<T> = std::result::Result<T, ControlConfigLoadError>;

fn get_xml_file_path<S: AsRef<str>>(name: S) -> Result<PathBuf> {
    let name = name.as_ref();
    ensure!(!name.contains(&INVALID_CHARS[..]), InvalidChars { name });

    let mut fan_config_path = CONTROL_CONFIGS_DIR_PATH.join(name);
    fan_config_path.set_extension("xml");

    Ok(fan_config_path)
}

/// Loads the fan control configuration directly from configs folder.
pub(crate) fn load_control_config<S: AsRef<str>>(name: S) -> Result<FanControlConfigV2> {
    info!("Loading fan control configuration '{}'", name.as_ref());

    let path = get_xml_file_path(name.as_ref())?;
    let mut config_file = File::open(path).context(Loading {
        name: name.as_ref(),
    })?;

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).context(Loading {
        name: name.as_ref(),
    })?;

    let c = xml_from_str::<XmlFanControlConfigV2>(&buf)
        .context(ControlXmlDeserialize {
            name: name.as_ref(),
        })?
        .into();

    Ok(c)
}

/// Test if the fan control config provided can be loaded.
pub(crate) fn test_load_control_config<S: AsRef<str>>(name: S, check_config: bool) -> Result<()> {
    info!("Testing fan control configuration '{}'", name.as_ref());

    let path = get_xml_file_path(name.as_ref())?;

    File::open(path)
        .context(Loading {
            name: name.as_ref(),
        })
        .and_then(|mut f| {
            let mut buf = String::new();
            f.read_to_string(&mut buf).context(Loading {
                name: name.as_ref(),
            })?;

            let c = xml_from_str::<XmlFanControlConfigV2>(&buf)
                .context(ControlXmlDeserialize {
                    name: name.as_ref(),
                })?
                .into();

            if !check_config {
                return Ok(());
            }

            check_control_config(&c).context(Check {
                name: name.as_ref(),
            })
        })
        .map(|_| {})
}
