/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use quick_xml::de::from_str as xml_from_str;
use snafu::{ResultExt, Snafu};

use crate::constants::CONTROL_CONFIGS_DIR_PATH;
use crate::constants::ROOT_CONFIG_PATH;
use crate::nbfc::{check_control_config, CheckControlConfigError, FanControlConfigV2};

#[derive(Debug, Snafu)]
pub(crate) enum ControlConfigLoadError {
    #[snafu(display(
        "An IO error occured while trying to load fan control config `{}`: {}",
        name.display(),
        source
    ))]
    Loading {
        name: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Error occured while deserializing XML: {}", source))]
    ControlXmlDeserialize { source: quick_xml::DeError },

    #[snafu(display("Error occured while checking control config at `{}`: {}", name.display() , source))]
    Check {
        name: PathBuf,
        source: CheckControlConfigError,
    },
}

/// Loads the fan control configuration directly from configs folder.
pub(crate) fn load_control_config<P: AsRef<Path>>(
    name: P,
) -> Result<FanControlConfigV2, ControlConfigLoadError> {
    let mut fan_config_path = ROOT_CONFIG_PATH
        .join(&*CONTROL_CONFIGS_DIR_PATH)
        .join(name.as_ref());
    fan_config_path.set_extension("xml");

    let mut config_file = File::open(fan_config_path).context(Loading {
        name: name.as_ref(),
    })?;

    let mut buf = String::new();
    config_file.read_to_string(&mut buf).context(Loading {
        name: name.as_ref(),
    })?;

    let c = xml_from_str::<FanControlConfigV2>(&buf).context(ControlXmlDeserialize {})?;
    check_control_config(&c).context(Check {
        name: name.as_ref(),
    })?;
    Ok(c)
}

/// Test if the fan control config provided can be loaded.
pub(crate) fn test_load_control_config<P: AsRef<Path>>(
    name: P,
) -> Result<(), ControlConfigLoadError> {
    let mut fan_config_path = ROOT_CONFIG_PATH
        .join(&*CONTROL_CONFIGS_DIR_PATH)
        .join(name.as_ref());
    fan_config_path.set_extension("xml");

    File::open(fan_config_path)
        .context(Loading {
            name: name.as_ref(),
        })
        .map(|_| ())
}
