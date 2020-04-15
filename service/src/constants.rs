/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dbus_crate::strings::{BusName, Path as DBusPath};
use once_cell::sync::Lazy;

use std::path::{Path, PathBuf};

pub const OBJ_PATH_STR: &str = "/com/musikid/fancy";
pub const BUS_NAME_STR: &str = "com.musikid.fancy";

pub static BUS_NAME: Lazy<BusName> = Lazy::new(|| BusName::new(BUS_NAME_STR).unwrap());
pub static DBUS_PATH: Lazy<DBusPath> = Lazy::new(|| DBusPath::new(OBJ_PATH_STR).unwrap());

pub static ROOT_CONFIG_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/etc/fancy"));
pub static CONFIG_FILE_PATH: Lazy<PathBuf> = Lazy::new(|| ROOT_CONFIG_PATH.join("config.toml"));
pub static NBFC_SETTINGS_PATH: Lazy<&Path> =
    Lazy::new(|| Path::new("/etc/NbfcService/NbfcServiceSettings.xml"));
pub static CONTROL_CONFIGS_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| ROOT_CONFIG_PATH.join("configs"));

pub static EC_SYS_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/sys/kernel/debug/ec/ec0/io"));
pub static ACPI_EC_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/ec"));
pub static PORT_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/port"));
