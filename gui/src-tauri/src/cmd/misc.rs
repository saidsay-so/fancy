use std::sync::Arc;

use quick_xml::de::from_str as xml_from_str;
use tauri::async_runtime::RwLock;
use tauri::{AppHandle, Manager};
use tokio::fs::{read_dir, read_to_string};

use super::dbus::ConfigInfo;
use super::CmdResult;
use crate::error::{Error, ErrorEvent, JsError};
use crate::State;
use nbfc_config::{FanControlConfigV2, XmlFanControlConfigV2};

#[tauri::command]
pub async fn get_model(state: tauri::State<'_, Arc<RwLock<State<'_>>>>) -> CmdResult<String> {
  let state = state.read().await;
  Ok(state.model.clone())
}

#[tauri::command]
pub async fn get_configs_list(app: AppHandle) -> CmdResult<Vec<ConfigInfo>> {
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
