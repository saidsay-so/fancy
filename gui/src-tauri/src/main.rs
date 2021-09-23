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
use tauri::async_runtime::RwLock;
use tauri::Manager;

use std::convert::AsRef;
use strum::{AsRefStr, Display};

macro_rules! zbus_conn_try {
  ($state: expr, $app: expr, $conn: expr) => {
    match $conn {
      Ok(c) => c,
      Err(e) => {
        $app
          .emit_all(
            ErrorEvent::ConnectionError.as_ref(),
            JsError::new(Error::ConnectionRefused(e.to_string()).to_string(), true),
          )
          .unwrap();
        $state.write().await.set_connection_error(e);
        return;
      }
    }
  };
}

macro_rules! zbus_changes_try {
  ($state: expr, $app: expr, $conn: expr) => {
    match $conn {
      Ok(c) => c,
      Err(e) => {
        $app
          .emit_all(
            ErrorEvent::ProxyError.as_ref(),
            JsError::new(Error::ChangesDBusError(e).to_string(), true),
          )
          .unwrap();
        return;
      }
    }
  };
}

#[derive(Display, AsRefStr, Debug)]
#[strum(serialize_all = "snake_case")]
enum Changes {
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
        let conn = zbus_conn_try!(state, app, zbus::azync::Connection::system().await);
        let proxy = zbus_conn_try!(
          state,
          app,
          AsyncFancyProxy::builder(&conn)
            .cache_properties(false)
            .build()
            .await
        );
        state.write().await.set_proxy(proxy);

        let signal_conn = zbus_conn_try!(state, app, zbus::azync::Connection::system().await);
        let changes_proxy = zbus_conn_try!(state, app, AsyncFancyProxy::new(&signal_conn).await);
        let mut target_changes = changes_proxy
          .receive_target_fans_speeds_changed()
          .await
          .fuse();
        let mut config_changes = changes_proxy.receive_config_changed().await.fuse();
        let mut auto_changes = changes_proxy.receive_auto_changed().await.fuse();

        {
          let mut state = state.write().await;
          state.config = zbus_changes_try!(state, app, changes_proxy.config().await);
          state.poll_interval = zbus_changes_try!(state, app, changes_proxy.poll_interval().await);
        }

        loop {
          select! {
            t = target_changes.select_next_some() => {
              if let Some(t) = t {
                let target: Vec<f64> = t.try_into().unwrap();
                app.emit_all(Changes::TargetSpeedsChange.as_ref(), target).unwrap();
              }
            },
            c = config_changes.select_next_some() => {
              if let Some(c) = c {
                let config: String = c.try_into().unwrap();
                let mut state = state.write().await;
                state.config = zbus_changes_try!(state, app, changes_proxy.config().await);
                state.poll_interval = zbus_changes_try!(state, app, changes_proxy.poll_interval().await);
                app.emit_all(Changes::ConfigChange.as_ref(), config).unwrap();
              }
            },
            a = auto_changes.select_next_some() => {
              if let Some(a) = a {
                let auto: bool = a.try_into().unwrap();
                app.emit_all(Changes::AutoChange.as_ref(), auto).unwrap();
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
      get_critical,
      get_names,
      get_poll_interval,
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
