#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
mod error;
mod interface;
mod state;

use cmd::*;
use error::{Error, ErrorEvent, JsError};
use interface::*;
use state::*;

use std::convert::TryInto;
use std::sync::Arc;

use futures::{select, StreamExt};
use strum::AsRefStr;
use tauri::async_runtime::RwLock;
use tauri::Manager;
use tokio::fs::read_to_string;

use std::convert::AsRef;

macro_rules! zbus_conn_try {
  ($state: expr, $app: expr, $conn: expr) => {
    match $conn {
      Ok(c) => c,
      Err(e) => {
        let js_err = Error::ConnectionRefused(e.to_string());
        $app
          .emit_all(
            ErrorEvent::ConnectionError.as_ref(),
            JsError::new(js_err.to_string(), js_err.as_ref().to_string(), true),
          )
          .unwrap();
        $state.write().await.set_connection_error(e);
        return;
      }
    }
  };
}

macro_rules! zbus_proxy_try {
  ($state: expr, $app: expr, $conn: expr) => {
    match $conn {
      Ok(c) => c,
      Err(e) => {
        let e = Error::ChangesDBusError(e);
        $app
          .emit_all(
            ErrorEvent::ProxyError.as_ref(),
            JsError::new(e.to_string(), e.as_ref().to_string(), true),
          )
          .unwrap();
        return;
      }
    }
  };
}

#[derive(AsRefStr, Debug)]
#[strum(serialize_all = "snake_case")]
enum ChangesEvent {
  TargetSpeedsChange,
  ConfigChange,
  AutoChange,
}

fn main() {
  let state = State::new();
  let state = Arc::from(RwLock::from(state));

  let state = state.clone();
  tauri::Builder::default()
    .manage(state.clone())
    .setup(move |app| {
      let app = app.handle();
      let state = state.clone();

      tauri::async_runtime::spawn(async move {
        let conn = zbus_conn_try!(state, app, zbus::Connection::system().await);
        let proxy = zbus_proxy_try!(
          state,
          app,
          FancyProxy::builder(&conn)
            .cache_properties(false)
            .build()
            .await
        );

        let changes_proxy = zbus_proxy_try!(state, app, FancyProxy::new(&conn).await);
        let mut target_changes = changes_proxy
          .receive_target_fans_speeds_changed()
          .await
          .fuse();
        let mut config_changes = changes_proxy.receive_config_changed().await.fuse();
        let mut auto_changes = changes_proxy.receive_auto_changed().await.fuse();

        {
          let mut state = state.write().await;
          state.set_proxy(proxy);
          // Get computer model
          state.model = read_to_string("/sys/devices/virtual/dmi/id/product_name")
            .await
            .unwrap();
          state.config = zbus_proxy_try!(state, app, changes_proxy.config().await);
        }

        loop {
          select! {
            t = target_changes.select_next_some() => {
              if let Some(t) = t {
                let target: Vec<f64> = t.try_into().unwrap();
                app.emit_all(ChangesEvent::TargetSpeedsChange.as_ref(), target).unwrap();
              }
            },
            c = config_changes.select_next_some() => {
              if let Some(c) = c {
                let config: String = c.try_into().unwrap();
                let mut state = state.write().await;
                state.config = zbus_proxy_try!(state, app, changes_proxy.config().await);
                app.emit_all(ChangesEvent::ConfigChange.as_ref(), config).unwrap();
              }
            },
            a = auto_changes.select_next_some() => {
              if let Some(a) = a {
                let auto: bool = a.try_into().unwrap();
                app.emit_all(ChangesEvent::AutoChange.as_ref(), auto).unwrap();
              }
            }
          }
        }
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Getters
      get_auto,
      get_config,
      get_configs_list,
      get_critical,
      get_model,
      get_names,
      get_poll_interval,
      get_speeds,
      get_target_speeds,
      get_temps,
      // Setters
      set_auto,
      set_config,
      set_target_speed,
      // Misc
      restart
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
