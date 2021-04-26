/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use once_cell::sync::Lazy;
use quick_xml::de::from_str as xml_from_str;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use toml::{de::from_str as toml_from_str, ser::to_string_pretty};

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use crate::constants::ROOT_CONFIG_PATH;
use crate::nbfc::NbfcServiceSettings;

static EC_SYS_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/sys/kernel/debug/ec/ec0/io"));
static ACPI_EC_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/ec"));
static PORT_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/port"));
static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| ROOT_CONFIG_PATH.join("config.toml"));
static NBFC_SETTINGS_PATH: Lazy<&Path> =
    Lazy::new(|| Path::new("/etc/NbfcService/NbfcServiceSettings.xml"));

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
/// Describe the way to access to the EC.
pub(crate) enum ECAccessMode {
    /// Access to the EC using the `/dev/port` file.
    RawPort,
    /// Access to the EC using the module `acpi_ec` (`/dev/ec`).
    AcpiEC,
    /// Access to the EC using the module `ec_sys` with `write_support=1`.
    ECSys,
    /// Determine the way to access to the EC at run.
    Either,
}

/// Get the device path from `ECAccessMode`.
impl ECAccessMode {
    /// Get path corresponding to the access mode.
    ///
    /// # Panics
    ///
    /// Panic if the value is [Either](#enum.ECAccessMode) and it's not possible to get an access to the EC.
    pub fn to_path(&self) -> &'static Path {
        match self {
            ECAccessMode::RawPort => *PORT_DEV_PATH,
            ECAccessMode::AcpiEC => *ACPI_EC_DEV_PATH,
            ECAccessMode::ECSys => *EC_SYS_DEV_PATH,
            ECAccessMode::Either => {
                if PORT_DEV_PATH.exists() {
                    *PORT_DEV_PATH
                } else if ACPI_EC_DEV_PATH.exists() {
                    *ACPI_EC_DEV_PATH
                } else if EC_SYS_DEV_PATH.exists() {
                    *EC_SYS_DEV_PATH
                } else {
                    panic!("No module for access to the EC is available")
                }
            }
        }
    }
}
impl Default for ECAccessMode {
    fn default() -> Self {
        ECAccessMode::Either
    }
}
impl From<&Path> for ECAccessMode {
    fn from(s: &Path) -> Self {
        match s {
            _s if _s == *PORT_DEV_PATH => ECAccessMode::RawPort,
            _s if _s == *ACPI_EC_DEV_PATH => ECAccessMode::AcpiEC,
            _s if _s == *EC_SYS_DEV_PATH => ECAccessMode::ECSys,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
/// Describe how to get the temperature.
pub(crate) enum TempComputeMethod {
    /// Get the CPU sensor data only.
    CPUOnly,
    /// Compute the average from all valid sensors.
    AllSensors,
}
impl Default for TempComputeMethod {
    fn default() -> Self {
        TempComputeMethod::CPUOnly
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
/// Stores the service configuration which can be written to the disk.
pub(crate) struct ServiceConfig {
    pub ec_access_mode: ECAccessMode,
    pub selected_fan_config: String,
    pub auto: bool,
    pub target_fans_speeds: Vec<f64>,
    #[serde(default)]
    pub temp_compute: TempComputeMethod,
}
impl From<NbfcServiceSettings> for ServiceConfig {
    fn from(s: NbfcServiceSettings) -> Self {
        ServiceConfig {
            ec_access_mode: ECAccessMode::default(),
            selected_fan_config: s.selected_config_id,
            auto: true, // Doesn't have the same meaning as in NBFC
            target_fans_speeds: s.target_fan_speeds.iter().map(|s| *s as f64).collect(),
            temp_compute: TempComputeMethod::default(),
        }
    }
}

#[derive(Debug, Snafu)]
pub(crate) enum ServiceConfigSaveError {
    #[snafu(display(
        "An I/O error occured while trying to create the service configuration file: {}",
        source
    ))]
    CreateConfig { source: std::io::Error },

    #[snafu(display(
        "An I/O error occured while trying to save the service configuration: {}",
        source
    ))]
    SaveConfig { source: std::io::Error },
}

#[derive(Debug, Snafu)]
pub(crate) enum ServiceConfigLoadError {
    #[snafu(display(
        "An I/O error occured while trying to open the service configuration file: {}",
        source
    ))]
    OpenServiceConfig { source: std::io::Error },

    #[snafu(display(
        "An I/O error occured while trying to open the NBFC service configuration file: {}",
        source
    ))]
    OpenNbfcServiceConfig { source: std::io::Error },

    #[snafu(display(
        "An I/O error occured while trying to load the service configuration: {}",
        source
    ))]
    LoadService { source: std::io::Error },

    #[snafu(display(
        "Error occured while deserializing NBFC service configuration: {}",
        source
    ))]
    NbfcServiceXmlDeserialize { source: quick_xml::DeError },

    #[snafu(display("Error occured while deserializing service configuration: {}", source))]
    TomlDeserialize { source: toml::de::Error },

    #[snafu(display("There is no configuration available"))]
    NoConfig {},
}

impl ServiceConfig {
    /// Loads the `ServiceConfig` from the disk (Fancy or NBFC format).
    pub(crate) fn load_service_config() -> Result<Self, ServiceConfigLoadError> {
        let mut buf = String::new();

        if CONFIG_FILE_PATH.is_file() {
            File::open(&*CONFIG_FILE_PATH)
                .context(OpenServiceConfig {})?
                .read_to_string(&mut buf)
                .context(LoadService {})?;

            toml_from_str::<ServiceConfig>(&buf).context(TomlDeserialize {})
        } else if NBFC_SETTINGS_PATH.is_file() {
            File::open(*NBFC_SETTINGS_PATH)
                .context(OpenNbfcServiceConfig {})?
                .read_to_string(&mut buf)
                .context(LoadService {})?;

            xml_from_str::<NbfcServiceSettings>(&buf)
                .context(NbfcServiceXmlDeserialize {})
                .map(|e| e.into())
        } else {
            Err(ServiceConfigLoadError::NoConfig {})
        }
    }

    /// Save the `ServiceConfig` to the disk.
    pub(crate) fn save(&self) -> Result<(), ServiceConfigSaveError> {
        File::create(&*CONFIG_FILE_PATH)
            .context(CreateConfig {})?
            .write_all(to_string_pretty(self).unwrap().as_bytes())
            .context(SaveConfig {})
    }
}
