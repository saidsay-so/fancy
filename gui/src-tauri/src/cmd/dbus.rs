use serde::{Deserialize, Serialize};
use tauri::api::process::restart as tauri_restart;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager};

use super::CmdResult;
use crate::error::{generate_proxy_err, Error, JsError};
use crate::state::State;
use crate::ChangesEvent;
use nbfc_config::TemperatureThreshold;

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
  pub model: String,
  pub name: String,
  pub author: Option<String>,
  pub thresholds: HashMap<String, Vec<TemperatureThreshold>>,
}

#[tauri::command]
pub async fn get_config(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<String> {
  let state = state.read().await;
  if let Some(_) = &state.proxy {
    // We use a cache because it doesn't often change
    Ok(state.config.clone())
  } else {
    Err(generate_proxy_err(&state.proxy_state))
  }
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
pub async fn set_config(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
  config: String,
) -> CmdResult<()> {
  prop!(state, set_config, &config)
}

#[tauri::command]
pub async fn get_auto(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<bool> {
  prop!(state, auto)
}

#[tauri::command]
pub async fn get_critical(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<bool> {
  prop!(state, critical)
}

#[tauri::command]
pub async fn get_names(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<Vec<String>> {
  prop!(state, fans_names)
}

#[tauri::command]
pub async fn get_poll_interval(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<u64> {
  prop!(state, poll_interval)
}

#[tauri::command]
pub async fn get_speeds(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<Vec<f64>> {
  prop!(state, fans_speeds)
}

#[tauri::command]
pub async fn get_target_speeds(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<Vec<f64>> {
  prop!(state, target_fans_speeds)
}

#[tauri::command]
pub async fn get_temps(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
) -> CmdResult<HashMap<String, f64>> {
  prop!(state, temperatures)
}

#[tauri::command]
pub async fn set_auto(
  state: tauri::State<'_, Arc<RwLock<State<'_>>>>,
  auto: bool,
) -> CmdResult<()> {
  prop!(state, set_auto, auto)
}

#[tauri::command]
pub async fn set_target_speed(
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
pub fn restart() {
  tauri_restart()
}
