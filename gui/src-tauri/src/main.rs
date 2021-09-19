#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
mod error;
mod interface;
mod state;

use cmd::*;
use error::{Error, JsError};
use interface::*;
use state::*;

use std::convert::TryInto;
use std::sync::Arc;

use futures::{select, StreamExt};
use tauri::async_runtime::RwLock;
use tauri::Manager;

macro_rules! zbus_try {
  ($state: expr, $app: expr, $conn: expr) => {
    match $conn {
      Ok(c) => c,
      Err(e) => {
        $app
          .emit_all(
            "connection_error",
            JsError::new(Error::ConnectionRefused(e.to_string()).to_string(), true),
          )
          .unwrap();
        $state.write().await.set_connection_error(e);
        return;
      }
    }
  };
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
        let conn = zbus_try!(state, app, zbus::azync::Connection::system().await);
        let proxy = zbus_try!(
          state,
          app,
          AsyncFancyProxy::builder(&conn)
            .cache_properties(false)
            .build()
            .await
        );
        state.write().await.set_proxy(proxy);

        let signal_conn = zbus_try!(state, app, zbus::azync::Connection::system().await);
        let changes_proxy = zbus_try!(state, app, AsyncFancyProxy::new(&signal_conn).await);
        let mut target_changes = changes_proxy
          .receive_target_fans_speeds_changed()
          .await
          .fuse();
        let mut config_changes = changes_proxy.receive_config_changed().await.fuse();
        let mut auto_changes = changes_proxy.receive_auto_changed().await.fuse();

        {
          state.write().await.config = changes_proxy.config().await.unwrap();
          state.write().await.poll_interval = changes_proxy.poll_interval().await.unwrap();
        }

        loop {
          select! {
            t = target_changes.next() => {
              if let Some(Some(t)) = t {
                let target: Vec<f64> = t.try_into().unwrap();
                app.emit_all("target_speeds_change", target).unwrap();
              }
            },
            c = config_changes.next() => {
              if let Some(Some(c)) = c {
                let config: String = c.try_into().unwrap();
                state.write().await.config = config.clone();
                app.emit_all("config_change", config).unwrap();
              }
            },
            a = auto_changes.next() => {
              if let Some(Some(a)) = a {
                let auto: bool = a.try_into().unwrap();
                app.emit_all("auto_change", auto).unwrap();
              }
            },
            default => {}
          }
        }
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Getters
      get_auto,
      get_config,
      get_critical,
      get_names,
      get_speeds,
      get_target_speeds,
      get_temps,
      // Setters
      set_auto,
      set_target_speed,
      // Misc
      restart
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
