/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::config::nbfc_control::ControlConfigLoader;
use crate::config::service::{ECAccessMode, ServiceConfig, TempComputeMethod};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Default)]
/// This struct is shared between the **D-Bus** tree and the `main` function.
/// It stores the current state of the information provided through **D-Bus** and is used to save
/// the configuration.
pub(crate) struct State {
    pub ec_access_mode: RefCell<ECAccessMode>,
    pub fans_speeds: RefCell<Vec<f64>>,
    pub target_fans_speeds: RefCell<Vec<f64>>,
    pub manual_set_target_speeds: RefCell<bool>,
    /// Used when an error occured while trying to change the configuration.
    pub old_config: RefCell<Option<String>>,
    pub auto: RefCell<bool>,
    pub critical: RefCell<bool>,
    pub config: RefCell<String>,
    pub temps: RefCell<HashMap<String, f64>>,
    pub temp_compute: RefCell<TempComputeMethod>,
    pub poll_interval: RefCell<u64>,
    pub fans_names: RefCell<Vec<String>>,
    pub check_control_config: RefCell<bool>,
    pub config_loader: RefCell<ControlConfigLoader>,
}
impl From<ServiceConfig> for State {
    fn from(s: ServiceConfig) -> Self {
        State {
            ec_access_mode: RefCell::new(s.ec_access_mode),
            fans_speeds: RefCell::new(Vec::new()),
            target_fans_speeds: RefCell::new(s.target_fans_speeds),
            manual_set_target_speeds: RefCell::new(false),
            old_config: RefCell::new(None),
            auto: RefCell::new(s.auto),
            critical: RefCell::new(false),
            config: RefCell::new(s.selected_fan_config),
            temps: RefCell::new(HashMap::new()),
            temp_compute: RefCell::new(s.temp_compute),
            poll_interval: RefCell::new(0),
            fans_names: RefCell::new(Vec::new()),
            check_control_config: RefCell::new(false),
            config_loader: RefCell::new(ControlConfigLoader::new(Vec::new())),
        }
    }
}
impl State {
    pub fn as_service_config(&self) -> ServiceConfig {
        ServiceConfig {
            ec_access_mode: *self.ec_access_mode.borrow(),
            auto: *self.auto.borrow(),
            target_fans_speeds: self.target_fans_speeds.borrow().to_owned(),
            selected_fan_config: self.config.borrow().to_owned(),
            temp_compute: *self.temp_compute.borrow(),
            check_control_config: *self.check_control_config.borrow(),
        }
    }
}
