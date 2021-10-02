use quick_xml::de::from_str as xml_from_str;
use serde::{Deserialize, Serialize};
use tauri::api::process::restart as tauri_restart;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager};
use tokio::fs::{read_dir, read_to_string};

use crate::error::{generate_proxy_err, Error, ErrorEvent, JsError};
use crate::state::State;
use crate::ChangesEvent;
use nbfc_config::{FanControlConfigV2, TemperatureThreshold, XmlFanControlConfigV2};

use std::collections::HashMap;
use std::sync::Arc;

type CmdResult<T> = std::result::Result<T, JsError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
  model: String,
  name: String,
  author: Option<String>,
  thresholds: HashMap<String, Vec<TemperatureThreshold>>,
}

#[tauri::command]
pub(super) async fn get_config(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<String> {
  let state = state.read().await;
  if let Some(_) = &state.proxy {
    // We use a cache because it doesn't often change
    Ok(state.config.clone())
  } else {
    Err(generate_proxy_err(&state.proxy_state))
  }
}

#[tauri::command]
pub(super) async fn get_model(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<String> {
  let state = state.read().await;
  Ok(state.model.clone())
}

#[tauri::command]
pub(super) async fn get_configs_list(app: AppHandle) -> CmdResult<Vec<ConfigInfo>> {
  let mut list = vec![];
  let mut configs = read_dir("/etc/fancy/configs")
    .await
    .map_err(Error::IoError)
    .map_err(|e| JsError::new(e.to_string(), e.as_ref().to_string(), true))?;

  while let Some(file) = configs
    .next_entry()
    .await
    .map_err(Error::IoError)
    .map_err(|e| JsError::new(e.to_string(), e.as_ref().to_string(), true))?
  {
    let path = file.path();
    let config = read_to_string(&path)
      .await
      .map_err(Error::IoError)
      .map_err(|e| JsError::new(e.to_string(), e.as_ref().to_string(), true))?;

    let config = match xml_from_str::<XmlFanControlConfigV2>(&*config) {
      Ok(c) => c,
      Err(e) => {
        let e = Error::InvalidConfiguration(e, path.to_string_lossy().to_string());
        let e = JsError::new(e.to_string(), e.as_ref().to_string(), false);
        app
          .emit_all(ErrorEvent::DeserializeError.as_ref(), e)
          .unwrap();
        continue;
      }
    };
    let config = FanControlConfigV2::from(config);

    list.push(ConfigInfo {
      author: config.author,
      model: config.notebook_model,
      name: path.file_stem().unwrap().to_string_lossy().to_string(),
      thresholds: config
        .fan_configurations
        .into_iter()
        .scan(0, |i, mut fc| {
          *i += 1;
          fc.temperature_thresholds.sort_unstable();
          Some((
            fc.fan_display_name.unwrap_or(format!("Fan #{}", i)),
            fc.temperature_thresholds,
          ))
        })
        .collect(),
    });
  }

  list.sort_unstable_by_key(|c| c.name.clone());

  Ok(list)
}

macro_rules! prop {
  ($state: expr, $proxy_prop: tt) => {
    prop!($state, $proxy_prop,)
  };

  ($state: expr, $proxy_prop: tt, $( $arg: expr ),*) => {{
    let state = $state.read().await;
    if let Some(proxy) = &state.proxy {
      proxy
        .$proxy_prop($( $arg ),*)
        .await
        .map_err(|e| Error::CmdDBusError(e, stringify!($proxy_prop).to_string()))
        .map_err(|e| JsError::new((e).to_string(), e.as_ref().to_string(), true))
    } else {
      Err(generate_proxy_err(&state.proxy_state))
    }
  }};
}

#[tauri::command]
pub(super) async fn set_config(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
  config: String,
) -> CmdResult<()> {
  prop!(state, set_config, &config)
}

#[tauri::command]
pub(super) async fn get_auto(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<bool> {
  prop!(state, auto)
}

#[tauri::command]
pub(super) async fn get_critical(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<bool> {
  prop!(state, critical)
}

#[tauri::command]
pub(super) async fn get_names(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<Vec<String>> {
  prop!(state, fans_names)
}

#[tauri::command]
pub(super) async fn get_poll_interval(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<u64> {
  prop!(state, poll_interval)
}

#[tauri::command]
pub(super) async fn get_speeds(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<Vec<f64>> {
  prop!(state, fans_speeds)
}

#[tauri::command]
pub(super) async fn get_target_speeds(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<Vec<f64>> {
  prop!(state, target_fans_speeds)
}

#[tauri::command]
pub(super) async fn get_temps(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<HashMap<String, f64>> {
  prop!(state, temperatures)
}

#[tauri::command]
pub(super) async fn set_auto(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
  auto: bool,
) -> CmdResult<()> {
  prop!(state, set_auto, auto)
}

#[tauri::command]
pub(super) async fn set_target_speed(
  app: AppHandle,
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
  index: u8,
  speed: f64,
) -> CmdResult<()> {
  prop!(state, set_target_fan_speed, index, speed)?;

  // TODO: We send the event manually because it doesn't seem
  // to detect the one sent by the service
  let state = state.read().await;
  let proxy = &state.proxy.as_ref().unwrap();
  app
    .emit_all(
      ChangesEvent::TargetSpeedsChange.as_ref(),
      proxy
        .target_fans_speeds()
        .await
        .map_err(|e| Error::CmdDBusError(e, "target_fans_speeds".to_string()))
        .map_err(|e| JsError::new((e).to_string(), e.as_ref().to_string(), true))?,
    )
    .unwrap();
  Ok(())
}

#[tauri::command]
pub(super) fn restart() {
  tauri_restart()
}
