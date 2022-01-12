use std::collections::HashMap;

use async_std::fs::{read_to_string, write};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::ec_control::EcAccessMode;

const DEFAULT_CONFIG_PATH: &str = "/etc/fancy/config.toml";

#[derive(Debug, Snafu)]
pub enum ConfigError {
    #[snafu(display("could not load config: {}", source))]
    Load { source: async_std::io::Error },

    #[snafu(display("could not save config: {}", source))]
    Save { source: async_std::io::Error },

    #[snafu(display("could not serialize config: {}", source))]
    SerializeErr { source: toml::ser::Error },

    #[snafu(display("could not deserialize config: {}", source))]
    DeserializeErr { source: toml::de::Error },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub core: Core,
    pub fan_config: FanConfig,
    pub sensors: Sensors,
}

impl Config {
    pub async fn load_config() -> Result<Self, ConfigError> {
        let config = read_to_string(DEFAULT_CONFIG_PATH).await.context(LoadSnafu)?;
        toml::from_str(&config).context(DeserializeErrSnafu {})
    }

    pub async fn save_config(&self) -> Result<(), ConfigError> {
        let config = toml::to_string(&self).context(SerializeErrSnafu)?;
        write(DEFAULT_CONFIG_PATH, config).await.context(SaveSnafu)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Core {
    pub ec_access_mode: EcAccessMode,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FanConfig {
    pub selected_fan_configuration: String,
    pub target_speeds: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Sensors {
    pub only: SensorsFilter,
}

//TODO: Think of a better way to handle this
pub type SensorsFilter = HashMap<String, SensorFilter>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SensorFilter {
    pub inputs_filter: Vec<String>,
}
