/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::config::service::{ECAccessMode, ServiceConfig};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
/// This struct is shared between the **D-Bus** tree and the `main` function.
/// Some of the properties are in common with the `ServiceConfig`.
/// It stores the current state of the information provided through **D-Bus**.
pub(crate) struct State {
    pub ec_access_mode: RwLock<ECAccessMode>,
    pub fans_speeds: RwLock<HashMap<String, f64>>,
    pub target_fans_speeds: RwLock<Vec<f64>>,
    pub auto: RwLock<bool>,
    pub critical: RwLock<bool>,
    pub config: RwLock<String>,
    pub temps: RwLock<HashMap<String, f64>>,
}
impl From<ServiceConfig> for State {
    fn from(s: ServiceConfig) -> Self {
        State {
            ec_access_mode: RwLock::new(s.ec_access_mode),
            fans_speeds: RwLock::new(HashMap::new()),
            target_fans_speeds: RwLock::new(s.target_fans_speeds),
            auto: RwLock::new(s.auto),
            critical: RwLock::new(false),
            config: RwLock::new(s.selected_fan_config),
            temps: RwLock::new(HashMap::new()),
        }
    }
}
impl State {
    pub fn as_service_config(&self) -> ServiceConfig {
        ServiceConfig {
            ec_access_mode: *self.ec_access_mode.read().unwrap(),
            auto: *self.auto.read().unwrap(),
            target_fans_speeds: self.target_fans_speeds.read().unwrap().to_owned(),
            selected_fan_config: self.config.read().unwrap().to_owned(),
        }
    }
}
