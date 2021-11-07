/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains structures for serializing and deserializing NBFC configuration.
//! The XML variants are here to apply custom deserializing for XML format. They should be used as
//! middleware before saving/reading configs.

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[serde(from = "String")]
pub enum RegisterWriteMode {
    Set,
    And,
    Or,
}
impl From<String> for RegisterWriteMode {
    fn from(s: String) -> RegisterWriteMode {
        match s.as_str() {
            "Set" => RegisterWriteMode::Set,
            "And" => RegisterWriteMode::And,
            "Or" => RegisterWriteMode::Or,
            _ => unreachable!(),
        }
    }
}

impl Default for RegisterWriteMode {
    fn default() -> Self {
        RegisterWriteMode::Set
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
#[serde(from = "String")]
pub enum RegisterWriteOccasion {
    OnWriteFanSpeed,
    OnInitialization,
}
impl From<String> for RegisterWriteOccasion {
    fn from(s: String) -> RegisterWriteOccasion {
        match s.as_str() {
            "OnWriteFanSpeed" => RegisterWriteOccasion::OnWriteFanSpeed,
            "OnInitialization" => RegisterWriteOccasion::OnInitialization,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(from = "String")]
pub enum OverrideTargetOperation {
    Read,      /* = 1 */
    Write,     /* = 2 */
    ReadWrite, /* = 4 */
}
impl From<String> for OverrideTargetOperation {
    fn from(s: String) -> OverrideTargetOperation {
        match s.as_str() {
            "Read" => OverrideTargetOperation::Read,
            "Write" => OverrideTargetOperation::Write,
            "ReadWrite" => OverrideTargetOperation::ReadWrite,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct TemperatureThreshold {
    pub up_threshold: u8,
    pub down_threshold: u8,
    pub fan_speed: f32,
}
impl PartialOrd for TemperatureThreshold {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.down_threshold.cmp(&other.down_threshold))
    }
}
impl PartialEq for TemperatureThreshold {
    fn eq(&self, other: &Self) -> bool {
        self.down_threshold == other.down_threshold
    }
}
impl Eq for TemperatureThreshold {}
impl Ord for TemperatureThreshold {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.down_threshold.cmp(&other.down_threshold)
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct FanSpeedPercentageOverride {
    pub fan_speed_percentage: f32,
    pub fan_speed_value: u16,
    pub target_operation: Option<OverrideTargetOperation>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct RegisterWriteConfiguration {
    #[serde(skip)] // Deprecated
    pub write_mode: RegisterWriteMode,
    pub write_occasion: Option<RegisterWriteOccasion>,
    pub register: u8,
    pub value: u8,
    #[serde(default)]
    pub reset_required: bool,
    pub reset_value: Option<u8>,
    #[serde(skip)] // Deprecated
    pub reset_write_mode: Option<RegisterWriteMode>,
    pub description: Option<String>,
}
//NOTE: Even if the docs seems to say that there should be at least one threshold with 100,
// some configs don't even have one.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TemperatureThresholds {
    #[serde(rename = "TemperatureThreshold")]
    #[serde(default = "default_temperature_thresholds")]
    temperature_thresholds: Vec<TemperatureThreshold>,
}
impl Default for TemperatureThresholds {
    fn default() -> Self {
        TemperatureThresholds {
            temperature_thresholds: default_temperature_thresholds(),
        }
    }
}
fn default_temperature_thresholds() -> Vec<TemperatureThreshold> {
    vec![
        TemperatureThreshold {
            up_threshold: 0,
            down_threshold: 0,
            fan_speed: 0.0,
        },
        TemperatureThreshold {
            up_threshold: 50,
            down_threshold: 40,
            fan_speed: 100.0,
        },
    ]
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FanSpeedPercentageOverrides {
    #[serde(rename = "FanSpeedPercentageOverride")]
    fan_speed_percentage_overrides: Option<Vec<FanSpeedPercentageOverride>>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XmlFanConfiguration {
    read_register: u8,
    write_register: u8,
    min_speed_value: u16,
    max_speed_value: u16,
    #[serde(default)]
    independent_read_min_max_values: bool,
    #[serde(default)]
    min_speed_value_read: u16,
    #[serde(default)]
    max_speed_value_read: u16,
    #[serde(default)]
    reset_required: bool,
    fan_speed_reset_value: Option<u16>,
    fan_display_name: Option<String>,
    #[serde(default)]
    temperature_thresholds: TemperatureThresholds,
    fan_speed_percentage_overrides: Option<FanSpeedPercentageOverrides>,
}

impl From<FanConfiguration> for XmlFanConfiguration {
    fn from(f: FanConfiguration) -> Self {
        XmlFanConfiguration {
            read_register: f.read_register,
            write_register: f.write_register,
            min_speed_value: f.min_speed_value,
            max_speed_value: f.max_speed_value,
            independent_read_min_max_values: f.independent_read_min_max_values,
            min_speed_value_read: f.min_speed_value_read,
            max_speed_value_read: f.max_speed_value_read,
            reset_required: f.reset_required,
            fan_speed_reset_value: f.fan_speed_reset_value,
            fan_display_name: f.fan_display_name,
            temperature_thresholds: TemperatureThresholds {
                temperature_thresholds: f.temperature_thresholds,
            },
            fan_speed_percentage_overrides: f.fan_speed_percentage_overrides.map(|o| {
                FanSpeedPercentageOverrides {
                    fan_speed_percentage_overrides: Some(o),
                }
            }),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct FanConfiguration {
    pub read_register: u8,
    pub write_register: u8,
    pub min_speed_value: u16,
    pub max_speed_value: u16,
    pub independent_read_min_max_values: bool,
    pub min_speed_value_read: u16,
    pub max_speed_value_read: u16,
    pub reset_required: bool,
    pub fan_speed_reset_value: Option<u16>,
    pub fan_display_name: Option<String>,
    pub temperature_thresholds: Vec<TemperatureThreshold>,
    pub fan_speed_percentage_overrides: Option<Vec<FanSpeedPercentageOverride>>,
}

impl From<XmlFanConfiguration> for FanConfiguration {
    fn from(f: XmlFanConfiguration) -> Self {
        FanConfiguration {
            read_register: f.read_register,
            write_register: f.write_register,
            min_speed_value: f.min_speed_value,
            max_speed_value: f.max_speed_value,
            independent_read_min_max_values: f.independent_read_min_max_values,
            min_speed_value_read: f.min_speed_value_read,
            max_speed_value_read: f.max_speed_value_read,
            reset_required: f.reset_required,
            fan_speed_reset_value: f.fan_speed_reset_value,
            fan_display_name: f.fan_display_name,
            temperature_thresholds: f.temperature_thresholds.temperature_thresholds,
            fan_speed_percentage_overrides: f
                .fan_speed_percentage_overrides
                .and_then(|o| o.fan_speed_percentage_overrides),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct FanConfigurations {
    #[serde(rename = "FanConfiguration")]
    fan_configurations: Vec<XmlFanConfiguration>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RegisterWriteConfigurations {
    #[serde(rename = "RegisterWriteConfiguration")]
    register_write_configurations: Option<Vec<RegisterWriteConfiguration>>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XmlFanControlConfigV2 {
    notebook_model: String,
    author: Option<String>,
    #[serde(default = "default_poll_interval")]
    ec_poll_interval: u64,
    read_write_words: bool,
    #[serde(default = "default_critic_temp")]
    critical_temperature: u8,
    fan_configurations: FanConfigurations,
    register_write_configurations: RegisterWriteConfigurations,
}
fn default_poll_interval() -> u64 {
    100
}
fn default_critic_temp() -> u8 {
    70
}

impl From<FanControlConfigV2> for XmlFanControlConfigV2 {
    fn from(f: FanControlConfigV2) -> Self {
        XmlFanControlConfigV2 {
            notebook_model: f.notebook_model,
            author: f.author,
            ec_poll_interval: f.ec_poll_interval,
            read_write_words: f.read_write_words,
            critical_temperature: f.critical_temperature,
            fan_configurations: FanConfigurations {
                fan_configurations: f
                    .fan_configurations
                    .into_iter()
                    .map(XmlFanConfiguration::from)
                    .collect(),
            },
            register_write_configurations: RegisterWriteConfigurations {
                register_write_configurations: f.register_write_configurations,
            },
        }
    }
}

#[derive(PartialEq, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct FanControlConfigV2 {
    pub notebook_model: String,
    pub author: Option<String>,
    pub ec_poll_interval: u64,
    pub read_write_words: bool,
    pub critical_temperature: u8,
    pub fan_configurations: Vec<FanConfiguration>,
    pub register_write_configurations: Option<Vec<RegisterWriteConfiguration>>,
}

impl From<XmlFanControlConfigV2> for FanControlConfigV2 {
    fn from(f: XmlFanControlConfigV2) -> Self {
        FanControlConfigV2 {
            notebook_model: f.notebook_model,
            author: f.author,
            ec_poll_interval: f.ec_poll_interval,
            read_write_words: f.read_write_words,
            critical_temperature: f.critical_temperature,
            fan_configurations: f
                .fan_configurations
                .fan_configurations
                .into_iter()
                .map(FanConfiguration::from)
                .collect(),
            register_write_configurations: f
                .register_write_configurations
                .register_write_configurations,
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct TargetFanSpeeds {
    #[serde(rename = "float")]
    target_fan_speeds: Vec<f32>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct XmlNbfcServiceSettings {
    #[serde(default)]
    settings_version: usize,
    selected_config_id: String,
    #[serde(default)]
    autostart: bool,
    #[serde(default)]
    read_only: bool,
    target_fan_speeds: TargetFanSpeeds,
}

impl From<NbfcServiceSettings> for XmlNbfcServiceSettings {
    fn from(s: NbfcServiceSettings) -> Self {
        XmlNbfcServiceSettings {
            settings_version: s.settings_version,
            selected_config_id: s.selected_config_id,
            autostart: s.autostart,
            read_only: s.read_only,
            target_fan_speeds: TargetFanSpeeds {
                target_fan_speeds: s.target_fan_speeds,
            },
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct NbfcServiceSettings {
    #[serde(default)]
    pub settings_version: usize,
    pub selected_config_id: String,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub read_only: bool,
    pub target_fan_speeds: Vec<f32>,
}

impl From<XmlNbfcServiceSettings> for NbfcServiceSettings {
    fn from(s: XmlNbfcServiceSettings) -> Self {
        NbfcServiceSettings {
            settings_version: s.settings_version,
            selected_config_id: s.selected_config_id,
            autostart: s.autostart,
            read_only: s.read_only,
            target_fan_speeds: s.target_fan_speeds.target_fan_speeds,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckControlConfigError {
    FanConfigurationsNotEmpty,
    MaxFanSpeedThresholdRequired,
    NoDuplicateTemperatureUpThresholds,
    UpThresholdMayNotBeLowerThanDownThreshold,
    UpThresholdsMustBeLowerThanCriticalTemperature,
}
impl std::fmt::Display for CheckControlConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CheckControlConfigError::FanConfigurationsNotEmpty =>
                    "There should be at least one fan configuration",
                CheckControlConfigError::MaxFanSpeedThresholdRequired =>
                    "There should be at least one threshold with the maximum fan speed",
                CheckControlConfigError::NoDuplicateTemperatureUpThresholds =>
                    "There shouldn't be any duplicate up thresholds",
                CheckControlConfigError::UpThresholdMayNotBeLowerThanDownThreshold =>
                    "Up threshold can't be lower than down threshold",
                CheckControlConfigError::UpThresholdsMustBeLowerThanCriticalTemperature =>
                    "Up threshold must be lower than critical temperature",
            }
        )
    }
}
impl std::error::Error for CheckControlConfigError {}

pub fn check_control_config(c: &FanControlConfigV2) -> Result<(), CheckControlConfigError> {
    // This error should never be reached since `serde` enforce that at least one fan configuration should be here
    if c.fan_configurations.is_empty() {
        return Err(CheckControlConfigError::FanConfigurationsNotEmpty);
    }

    if !c.fan_configurations.iter().all(|f| {
        f.temperature_thresholds
            .iter()
            .any(|t| (t.fan_speed - 100.0).abs() < f32::EPSILON)
    }) {
        return Err(CheckControlConfigError::MaxFanSpeedThresholdRequired);
    }

    if c.fan_configurations.iter().any(|f| {
        (1..f.temperature_thresholds.len())
            .any(|i| f.temperature_thresholds[i..].contains(&f.temperature_thresholds[i - 1]))
    }) {
        return Err(CheckControlConfigError::NoDuplicateTemperatureUpThresholds);
    }

    if c.fan_configurations.iter().any(|f| {
        f.temperature_thresholds
            .iter()
            .any(|t| t.up_threshold < t.down_threshold)
    }) {
        return Err(CheckControlConfigError::UpThresholdMayNotBeLowerThanDownThreshold);
    }

    if c.fan_configurations.iter().any(|f| {
        f.temperature_thresholds
            .iter()
            .any(|t| t.up_threshold > c.critical_temperature as u8)
    }) {
        return Err(CheckControlConfigError::UpThresholdsMustBeLowerThanCriticalTemperature);
    }

    Ok(())
}

//TODO: More tests
#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;

    #[test]
    fn config_xml_parse_all_fields() {
        let config = r##"
        <?xml version="1.0"?>
    <FanControlConfigV2 xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <NotebookModel>HP Envy X360 13-ag0xxx Ryzen-APU</NotebookModel>
    <Author>Daniel Andersen</Author>
    <EcPollInterval>1000</EcPollInterval>
    <ReadWriteWords>true</ReadWriteWords>
    <CriticalTemperature>90</CriticalTemperature>
    <FanConfigurations>
        <FanConfiguration>
            <ReadRegister>149</ReadRegister>
            <WriteRegister>148</WriteRegister>
            <MinSpeedValue>175</MinSpeedValue>
            <MaxSpeedValue>70</MaxSpeedValue>
            <IndependentReadMinMaxValues>false</IndependentReadMinMaxValues>
            <MinSpeedValueRead>0</MinSpeedValueRead>
            <MaxSpeedValueRead>0</MaxSpeedValueRead>
            <ResetRequired>false</ResetRequired>
            <FanSpeedResetValue>255</FanSpeedResetValue>
            <FanDisplayName>CPU fan</FanDisplayName>
            <TemperatureThresholds>
                <TemperatureThreshold>
                <UpThreshold>0</UpThreshold>
                <DownThreshold>0</DownThreshold>
                <FanSpeed>0</FanSpeed>
                </TemperatureThreshold>
                <TemperatureThreshold>
                <UpThreshold>60</UpThreshold>
                <DownThreshold>48</DownThreshold>
                <FanSpeed>10</FanSpeed>
                </TemperatureThreshold>
                <TemperatureThreshold>
                <UpThreshold>63</UpThreshold>
                <DownThreshold>55</DownThreshold>
                <FanSpeed>20</FanSpeed>
                </TemperatureThreshold>
                <TemperatureThreshold>
                <UpThreshold>66</UpThreshold>
                <DownThreshold>59</DownThreshold>
                <FanSpeed>50</FanSpeed>
                </TemperatureThreshold>
                <TemperatureThreshold>
                <UpThreshold>68</UpThreshold>
                <DownThreshold>63</DownThreshold>
                <FanSpeed>70</FanSpeed>
                </TemperatureThreshold>
                <TemperatureThreshold>
                <UpThreshold>71</UpThreshold>
                <DownThreshold>67</DownThreshold>
                <FanSpeed>100</FanSpeed>
                </TemperatureThreshold>
            </TemperatureThresholds>
            <FanSpeedPercentageOverrides>
                <FanSpeedPercentageOverride>
                <FanSpeedPercentage>0</FanSpeedPercentage>
                <FanSpeedValue>255</FanSpeedValue>
                <TargetOperation>ReadWrite</TargetOperation>
                </FanSpeedPercentageOverride>
            </FanSpeedPercentageOverrides>
        </FanConfiguration>
    </FanConfigurations>
    <RegisterWriteConfigurations>
        <RegisterWriteConfiguration>
        <WriteMode>Set</WriteMode>
        <WriteOccasion>OnInitialization</WriteOccasion>
        <Register>147</Register>
        <Value>20</Value>
        <ResetRequired>true</ResetRequired>
        <ResetValue>4</ResetValue>
        <ResetWriteMode>Set</ResetWriteMode>
        <Description>Set EC to manual control</Description>
        </RegisterWriteConfiguration>
    </RegisterWriteConfigurations>
    </FanControlConfigV2>
        "##;
        let parsed_config: FanControlConfigV2 =
            from_str::<XmlFanControlConfigV2>(config).unwrap().into();
        let excepted_config = FanControlConfigV2 {
            notebook_model: "HP Envy X360 13-ag0xxx Ryzen-APU".to_string(),
            author: Some("Daniel Andersen".to_string()),
            ec_poll_interval: 1000,
            read_write_words: true,
            critical_temperature: 90,
            fan_configurations: [FanConfiguration {
                read_register: 149,
                write_register: 148,
                min_speed_value: 175,
                max_speed_value: 70,
                independent_read_min_max_values: false,
                min_speed_value_read: 0,
                max_speed_value_read: 0,
                reset_required: false,
                fan_speed_reset_value: Some(255),
                fan_display_name: Some("CPU fan".to_string()),
                temperature_thresholds: [
                    TemperatureThreshold {
                        up_threshold: 0,
                        down_threshold: 0,
                        fan_speed: 0.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 60,
                        down_threshold: 48,
                        fan_speed: 10.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 63,
                        down_threshold: 55,
                        fan_speed: 20.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 66,
                        down_threshold: 59,
                        fan_speed: 50.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 68,
                        down_threshold: 63,
                        fan_speed: 70.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 71,
                        down_threshold: 67,
                        fan_speed: 100.0,
                    },
                ]
                .to_vec(),
                fan_speed_percentage_overrides: Some(
                    [FanSpeedPercentageOverride {
                        fan_speed_percentage: 0.0,
                        fan_speed_value: 255,
                        target_operation: Some(OverrideTargetOperation::ReadWrite),
                    }]
                    .to_vec(),
                ),
            }]
            .to_vec(),
            register_write_configurations: Some(
                [RegisterWriteConfiguration {
                    write_mode: RegisterWriteMode::Set,
                    write_occasion: Some(RegisterWriteOccasion::OnInitialization),
                    register: 147,
                    value: 20,
                    reset_required: true,
                    reset_value: Some(4),
                    reset_write_mode: None,
                    description: Some("Set EC to manual control".to_string()),
                }]
                .to_vec(),
            ),
        };
        assert!(parsed_config == excepted_config);
    }

    #[test]
    fn config_parse_not_all_fields() {
        let config = r##"<?xml version="1.0"?>
<FanControlConfigV2 xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
  <NotebookModel>Aspire 1810TZ</NotebookModel>
  <EcPollInterval>3000</EcPollInterval>
  <ReadWriteWords>false</ReadWriteWords>
  <CriticalTemperature>70</CriticalTemperature>
  <FanConfigurations>
    <FanConfiguration>
      <ReadRegister>85</ReadRegister>
      <WriteRegister>85</WriteRegister>
      <MinSpeedValue>1</MinSpeedValue>
      <MaxSpeedValue>0</MaxSpeedValue>
      <ResetRequired>true</ResetRequired>
      <FanSpeedResetValue>0</FanSpeedResetValue>
      <TemperatureThresholds>
        <TemperatureThreshold>
          <UpThreshold>0</UpThreshold>
          <DownThreshold>0</DownThreshold>
          <FanSpeed>0</FanSpeed>
        </TemperatureThreshold>
        <TemperatureThreshold>
          <UpThreshold>56</UpThreshold>
          <DownThreshold>46</DownThreshold>
          <FanSpeed>100</FanSpeed>
        </TemperatureThreshold>
      </TemperatureThresholds>
      <FanSpeedPercentageOverrides>
        <FanSpeedPercentageOverride>
          <FanSpeedPercentage>0</FanSpeedPercentage>
          <FanSpeedValue>158</FanSpeedValue>
        </FanSpeedPercentageOverride>
      </FanSpeedPercentageOverrides>
    </FanConfiguration>
  </FanConfigurations>
  <RegisterWriteConfigurations />
</FanControlConfigV2>"##;
        let parsed_config = from_str::<XmlFanControlConfigV2>(config).unwrap();
        let parsed_config = FanControlConfigV2::from(parsed_config);
        let excepted_config = FanControlConfigV2 {
            notebook_model: "Aspire 1810TZ".to_string(),
            author: None,
            ec_poll_interval: 3000,
            read_write_words: false,
            critical_temperature: 70,
            fan_configurations: [FanConfiguration {
                read_register: 85,
                write_register: 85,
                min_speed_value: 1,
                max_speed_value: 0,
                independent_read_min_max_values: false,
                min_speed_value_read: 0,
                max_speed_value_read: 0,
                reset_required: true,
                fan_speed_reset_value: Some(0),
                fan_display_name: None,
                temperature_thresholds: [
                    TemperatureThreshold {
                        up_threshold: 0,
                        down_threshold: 0,
                        fan_speed: 0.0,
                    },
                    TemperatureThreshold {
                        up_threshold: 56,
                        down_threshold: 46,
                        fan_speed: 100.0,
                    },
                ]
                .to_vec(),
                fan_speed_percentage_overrides: Some(
                    [FanSpeedPercentageOverride {
                        fan_speed_percentage: 0.0,
                        fan_speed_value: 158,
                        target_operation: None,
                    }]
                    .to_vec(),
                ),
            }]
            .to_vec(),
            register_write_configurations: None,
        };
        assert_eq!(excepted_config, parsed_config);
    }

    #[test]
    fn all_configs() {
        std::fs::read_dir("nbfc_configs/Configs")
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| std::fs::read_to_string(e.path()).unwrap())
            .for_each(|e| {
                assert!(from_str::<XmlFanControlConfigV2>(&e).is_ok());
            });
    }

    const SETTINGS: &str = r##"<?xml version="1.0"?>
<NbfcServiceSettings xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <SettingsVersion>0</SettingsVersion>
  <SelectedConfigId>HP ENVY x360 Convertible 13-ag0xxx</SelectedConfigId>
  <Autostart>true</Autostart>
  <ReadOnly>false</ReadOnly>
  <TargetFanSpeeds>
    <float>0</float>
  </TargetFanSpeeds>
</NbfcServiceSettings>"##;

    #[test]
    fn settings_xml_parse() {
        let parsed_settings = from_str::<XmlNbfcServiceSettings>(SETTINGS).unwrap();
        let parsed_settings = NbfcServiceSettings::from(parsed_settings);
        let excepted_settings = NbfcServiceSettings {
            settings_version: 0,
            selected_config_id: "HP ENVY x360 Convertible 13-ag0xxx".to_string(),
            autostart: true,
            read_only: false,
            target_fan_speeds: [0.0].to_vec(),
        };

        assert_eq!(parsed_settings, excepted_settings);
    }
}
