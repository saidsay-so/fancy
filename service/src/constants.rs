/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use once_cell::sync::Lazy;

use std::path::{Path, PathBuf};

pub const OBJ_PATH_STR: &str = "/com/musikid/fancy";
pub const BUS_NAME_STR: &str = "com.musikid.fancy";
pub static ROOT_CONFIG_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/etc/fancy"));
pub static CONTROL_CONFIGS_DIR_PATH: Lazy<PathBuf> = Lazy::new(|| ROOT_CONFIG_PATH.join("configs"));
