/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use psutil::sensors::temperatures;
use snafu::Snafu;

use std::collections::HashMap;

const CPU_SENSORS_NAMES: &[&str] = &["coretemp", "k10temp"];

//NOTE: NVIDIA sensors don't always appear on the HWMON interface (when using the proprietary driver).
// The only consistent way to access to them seems to use the `nvidia-smi` cli.
const GPU_SENSORS_NAMES: &[&str] = &["amdgpu", "radeon", "nouveau"];
const ACPI_SENSORS_NAMES: &[&str] = &["acpitz"];
const NVME_SENSORS_NAMES: &[&str] = &["nvme"];
#[derive(Debug, Snafu)]
pub(crate) enum SensorError {
    #[snafu(display("Could not get access to a CPU sensor"))]
    NoCPUSensorFound,
}

#[derive(Debug)]
/// This structure holds temperatures of various sensors through simple categories.
pub(crate) struct Temperatures {
    pub cpu_temp: f64,
    pub gpu_temp: Option<f64>,
    //TODO: The following sensors should be implemented in another structure
    pub nvme_temp: Option<f64>,
    pub acpi_temp: Option<f64>,
}

impl Temperatures {
    //TODO: Find a way to hold some references to the sensors to not have to refresh anytime.
    //TODO: Manage errors
    /// Get the current temperatures.
    pub fn get_temps() -> Result<Self, SensorError> {
        let cpu_sensors: Vec<f64> = temperatures()
            .into_iter()
            .filter_map(|s| s.ok())
            .filter(|s| CPU_SENSORS_NAMES.iter().any(|&c| c.contains(s.unit())))
            .map(|s| s.current().celsius() as f64)
            .collect();
        if cpu_sensors.is_empty() {
            return Err(SensorError::NoCPUSensorFound {});
        }

        let gpu_sensors: Vec<f64> = temperatures()
            .into_iter()
            .filter_map(|s| s.ok())
            .filter(|s| GPU_SENSORS_NAMES.iter().any(|&c| c.contains(s.unit())))
            .map(|s| s.current().celsius() as f64)
            .collect();

        let acpi_sensors: Vec<f64> = temperatures()
            .into_iter()
            .filter_map(|s| s.ok())
            .filter(|s| ACPI_SENSORS_NAMES.iter().any(|&c| c.contains(s.unit())))
            .map(|s| s.current().celsius() as f64)
            .collect();

        let nvme_sensors: Vec<f64> = temperatures()
            .into_iter()
            .filter_map(|s| s.ok())
            .filter(|s| NVME_SENSORS_NAMES.iter().any(|&c| c.contains(s.unit())))
            .map(|s| s.current().celsius() as f64)
            .collect();

        Ok(Temperatures {
            cpu_temp: cpu_sensors.iter().fold(0f64, |a, s| a + s) / cpu_sensors.len() as f64,
            gpu_temp: if !gpu_sensors.is_empty() {
                Some(gpu_sensors.iter().fold(0f64, |a, s| a + s) / gpu_sensors.len() as f64)
            } else {
                None
            },
            acpi_temp: if !acpi_sensors.is_empty() {
                Some(acpi_sensors.iter().fold(0f64, |a, s| a + s) / acpi_sensors.len() as f64)
            } else {
                None
            },
            nvme_temp: if !nvme_sensors.is_empty() {
                Some(nvme_sensors.iter().fold(0f64, |a, s| a + s) / nvme_sensors.len() as f64)
            } else {
                None
            },
        })
    }

    pub fn update_map(&self, m: &mut HashMap<String, f64>) {
        m.insert("CPU".to_owned(), self.cpu_temp);
        if let Some(gpu_temp) = self.gpu_temp {
            m.insert("GPU".to_owned(), gpu_temp);
        }

        if let Some(acpi_temp) = self.acpi_temp {
            m.insert("ACPI".to_owned(), acpi_temp);
        }

        if let Some(nvme_temp) = self.nvme_temp {
            m.insert("NVME".to_owned(), nvme_temp);
        }
    }
}
