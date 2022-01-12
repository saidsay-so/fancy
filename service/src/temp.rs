use async_std::fs::{read_to_string, File, OpenOptions};
use async_std::path::Path;
use futures::{future, TryFutureExt, TryStreamExt};
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use snafu::{ensure, ResultExt, Snafu};

use async_std::io::prelude::*;
use async_std::{fs::read_dir, io::Read};
use futures::stream::{FuturesUnordered, StreamExt};
use std::fmt::Debug;
use std::os::unix::prelude::OsStrExt;

use crate::config::{SensorFilter, SensorsFilter};

const CPU_SENSORS_NAMES: &[&str] = &["coretemp", "k10temp"];

const SYSFS_HWMON_PATH: &str = "/sys/class/hwmon/";
const TEMP_PREFIX: &str = "temp";
const INPUT_SUFFIX: &str = "_input";
const LABEL_SUFFIX: &str = "_label";

//NOTE: NVIDIA sensors don't always appear on the HWMON interface (when using the proprietary driver).
// The only consistent way to access to them seems to use the `nvidia-smi` cli.
const GPU_SENSORS_NAMES: &[&str] = &["amdgpu", "radeon", "nouveau"];
const ACPI_SENSORS_NAMES: &[&str] = &["acpitz"];
const NVME_SENSORS_NAMES: &[&str] = &["nvme"];

#[derive(Debug, Snafu)]
pub(crate) enum SensorError {
    #[snafu(display("Could not get access to a CPU sensor"))]
    NoCPUSensorFound,

    #[snafu(display("Error while opening hwmon file: {}", source))]
    HwMonOpen { source: async_std::io::Error },

    #[snafu(display("Error while reading sensor: {}", source))]
    SensorErr { source: async_std::io::Error },
}

#[derive(Debug)]
struct Input<R> {
    label: String,
    handle: R,
    // previous_temp: f64,
    // resolution: Option<f64>,
}

impl<R: Read + Unpin + Debug> Input<R> {
    pub fn new(label: String, handle: R) -> Input<R> {
        Input { label, handle }
    }

    /*pub fn new_with_resolution(label: String, handle: R, resolution: f64) -> Self {
        let mut s = Self::new(label, handle);
        s.resolution = Some(resolution);
        s
    }*/

    pub async fn get_temp(&mut self) -> Result<f64, async_std::io::Error> {
        let mut value = String::with_capacity(8);
        self.handle.read_to_string(&mut value).await?;
        let value: f64 = value.parse().unwrap();
        //if let Some(resolution) = self.resolution {
        //if value - self.previous_temp >= resolution {
        //self.previous_temp = value;
        //Ok(Some(value))
        //} else {
        //Ok(None)
        //}
        //} else {
        //self.previous_temp = value;
        //Ok(Some(value))
        //}
        Ok(value / 1000.0)
    }
}
/*
#[derive(Debug, PartialEq, Eq)]
enum TempComputeMethod {
    Mean,
    Max,
}

impl Default for TempComputeMethod {
    fn default() -> Self {
        TempComputeMethod::Mean
    }
}*/

#[derive(Debug)]
struct Sensor<R> {
    name: String,
    inputs: Vec<Input<R>>,
    //compute_method: TempComputeMethod,
}

impl Sensor<File> {
    pub async fn from_hwmon_entry<P: AsRef<Path>>(
        hwmon_entry_path: P,
    ) -> Result<Self, SensorError> {
        let path = hwmon_entry_path.as_ref();
        let name = read_to_string(path.join("name"))
            .await
            .context(SensorErrSnafu {})?;
        let inputs = read_dir(path)
            .try_flatten_stream()
            .try_filter_map(|f| {
                let f = f.clone();
                async move {
                    let name = f.file_name();
                    let name = name.to_string_lossy();
                    let label_file_name = name.replace(INPUT_SUFFIX, LABEL_SUFFIX);
                    let mut label_path = f.path().clone();
                    label_path.set_file_name(label_file_name);

                    Ok(
                        (name.starts_with(TEMP_PREFIX) && name.ends_with(INPUT_SUFFIX))
                            .then(|| (f.path(), label_path)),
                    )
                }
            })
            .and_then(|(input_path, label_path)| async move {
                let label = read_to_string(label_path).await?;
                let handle = OpenOptions::new().read(true).open(input_path).await?;
                Ok(Input { label, handle })
            })
            .try_collect()
            .await
            .context(HwMonOpenSnafu {})?;

        Ok(Sensor::new(name, inputs))
    }
}

impl<R: Read + Unpin + Debug> Sensor<R> {
    pub fn new(name: String, inputs: Vec<Input<R>>) -> Self {
        Self {
            name,
            inputs,
            //compute_method: TempComputeMethod::default(),
        }
    }
    /*
    pub fn custom_compute(
        name: String,
        inputs: Vec<Input<R>>,
        compute_method: TempComputeMethod,
    ) -> Self {
        Self {
            name,
            inputs,
            compute_method,
        }
    }*/
    pub async fn get_temp(&mut self) -> Result<f64, async_std::io::Error> {
        /*match self.compute_method {
        TempComputeMethod::Max => {
            self.inputs
                .iter_mut()
                .map(|input| input.get_temp())
                .collect::<FuturesUnordered<_>>()
                .try_fold(0.0, |max, value| future::ok(value.max(max)))
                .await?
        }
        _ => {*/
        self.inputs
            .iter_mut()
            .map(|input| input.get_temp())
            .collect::<FuturesUnordered<_>>()
            .try_fold(0.0, |sum, value| future::ok(sum + value))
            .await
        /*}
        };*/
    }
}

#[derive(Debug)]
/// This structure holds temperatures of various sensors through simple categories.
pub(crate) struct Temperatures<R> {
    sensors: Vec<Sensor<R>>,
}

//TODO: Also filter inputs
impl Temperatures<File> {
    pub async fn new(filter: SensorsFilter) -> Result<Self, SensorError> {
        let sensors = read_dir(SYSFS_HWMON_PATH)
            .try_flatten_stream()
            .map(|res| res.context(SensorErrSnafu {}))
            .try_filter_map(move |f| {
                let filter = filter.clone();
                async move {
                    let path = f.path();
                    let root_hwmon_path = if path.join("device").exists().await {
                        path.join("device")
                    } else {
                        path.clone()
                    };
                    let first_temp_path =
                        root_hwmon_path.join(TEMP_PREFIX.to_string() + "1" + INPUT_SUFFIX);
                    let name = read_to_string(root_hwmon_path.join("name"))
                        .await
                        .context(SensorErrSnafu {})?;

                    Ok(
                        if first_temp_path.exists().await
                            && (filter.contains_key(&name) || filter.is_empty())
                        {
                            Some(root_hwmon_path)
                        } else {
                            None
                        },
                    )
                }
            })
            .and_then(|path| async move { Sensor::from_hwmon_entry(path).await })
            .try_collect::<Vec<_>>()
            .await?;

        ensure!(!sensors.is_empty(), NoCPUSensorFoundSnafu {});

        Ok(Self { sensors })
    }

    pub async fn get_temp(&mut self) -> Result<f64, SensorError> {
        /*match self.compute_method {
        TempComputeMethod::Max => {
            self.inputs
                .iter_mut()
                .map(|input| input.get_temp())
                .collect::<FuturesUnordered<_>>()
                .try_fold(0.0, |max, value| future::ok(value.max(max)))
                .await?
        }
        _ => {*/
        self.sensors
            .iter_mut()
            .map(|input| input.get_temp())
            .collect::<FuturesUnordered<_>>()
            .try_fold(0.0, |sum, value| future::ok(sum + value))
            .await
            .context(SensorErrSnafu {})
        /*}
        };*/
    }
}
