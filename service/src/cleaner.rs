/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::config::{nbfc_control::load_control_config, service::ServiceConfig};
use crate::ec_control::ECWriter;

use std::sync::{Arc, Mutex};

/// This function gets called when the main loop exits or a signal is caught. It resets all the registers.
pub(super) fn cleaner() {
    let serv_conf = ServiceConfig::load_service_config().unwrap();
    let c = load_control_config(serv_conf.selected_fan_config).unwrap();
    let path = serv_conf.ec_access_mode.to_path();
    let ec_dev = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();

    let mut writer = ECWriter::new(Arc::from(Mutex::from(ec_dev)));

    writer
        .refresh_config(
            c.read_write_words,
            c.register_write_configurations,
            &c.fan_configurations,
        )
        .unwrap();

    writer.reset(true).unwrap()
}
