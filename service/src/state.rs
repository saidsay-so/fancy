/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::config::service::{ECAccessMode, ServiceConfig};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Default)]
/// This struct is shared between the **D-Bus** tree and the `main` function.
/// Some of the properties are in common with the `ServiceConfig`.
/// It stores the current state of the information provided through **D-Bus**.
pub(crate) struct State {
    pub ec_access_mode: RefCell<ECAccessMode>,
    pub fans_speeds: RefCell<HashMap<String, f64>>,
    pub target_fans_speeds: RefCell<Vec<f64>>,
    pub auto: RefCell<bool>,
    pub critical: RefCell<bool>,
    pub config: RefCell<String>,
    pub temps: RefCell<HashMap<String, f64>>,
}
impl From<ServiceConfig> for State {
    fn from(s: ServiceConfig) -> Self {
        State {
            ec_access_mode: RefCell::new(s.ec_access_mode),
            fans_speeds: RefCell::new(HashMap::new()),
            target_fans_speeds: RefCell::new(s.target_fans_speeds),
            auto: RefCell::new(s.auto),
            critical: RefCell::new(false),
            config: RefCell::new(s.selected_fan_config),
            temps: RefCell::new(HashMap::new()),
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
        }
    }
}
