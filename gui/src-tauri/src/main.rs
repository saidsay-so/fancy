#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod interface;
mod state;
use interface::*;
use state::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use futures::{select, StreamExt};
use tauri::Manager;

#[tauri::command]
fn get_config(state: tauri::State<Arc<RwLock<State>>>) -> String {
  let state = state.read().unwrap();
  state.config.clone()
}

macro_rules! prop_getter {
  ($state: expr, $send_msg: path, $recv_msg: path) => {{
    $state.msg_sender.send($send_msg).unwrap();
    match $state.msg_receiver.recv().unwrap() {
      $recv_msg(t) => t,
      _ => unreachable!(),
    }
  }};
}

#[tauri::command]
fn get_poll_interval(state: tauri::State<Arc<RwLock<State>>>) -> u64 {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetPollInterval, Msg::PollInterval)
}

#[tauri::command]
fn get_temps(state: tauri::State<Arc<RwLock<State>>>) -> HashMap<String, f64> {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetTemps, Msg::Temps)
}

#[tauri::command]
fn get_speeds(state: tauri::State<Arc<RwLock<State>>>) -> Vec<f64> {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetSpeeds, Msg::Speeds)
}

#[tauri::command]
fn get_target_speeds(state: tauri::State<Arc<RwLock<State>>>) -> Vec<f64> {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetTargetSpeeds, Msg::TargetSpeeds)
}

#[tauri::command]
fn get_critical(state: tauri::State<Arc<RwLock<State>>>) -> bool {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetCritical, Msg::Critical)
}

#[tauri::command]
fn get_names(state: tauri::State<Arc<RwLock<State>>>) -> Vec<String> {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetNames, Msg::Names)
}

#[tauri::command]
fn get_auto(state: tauri::State<Arc<RwLock<State>>>) -> bool {
  let state = state.read().unwrap();
  prop_getter!(state, Msg::GetAuto, Msg::Auto)
}

#[tauri::command]
fn set_auto(state: tauri::State<Arc<RwLock<State>>>, auto: bool) {
  let state = state.read().unwrap();
  state.msg_sender.send(Msg::SetAuto(auto)).unwrap();
}

#[tauri::command]
fn set_target_speed(state: tauri::State<Arc<RwLock<State>>>, index: u8, speed: f64) {
  let state = state.read().unwrap();
  state
    .msg_sender
    .send(Msg::SetTargetSpeed(index, speed))
    .unwrap();
}

fn main() {
  let (client_send, backend_recv) = flume::bounded(1);
  let (backend_send, client_recv) = flume::bounded(1);
  let state = State::new(client_send, client_recv);
  let state = Arc::from(RwLock::from(state));

  let state = state.clone();
  let backend_recv = backend_recv.clone();
  let backend_send = backend_send.clone();
  tauri::Builder::default()
    .manage(state.clone())
    .setup(move |app| {
      let app = app.handle();
      let state = state.clone();
      let backend_recv = backend_recv.clone();
      let backend_send = backend_send.clone();

      tauri::async_runtime::spawn(async move {
        let conn = zbus::azync::Connection::system()
          .await
          .expect("could not create the connection");
        let signal_conn = zbus::azync::Connection::system().await.unwrap();
        let proxy = AsyncFancyProxy::builder(&conn)
          .cache_properties(false)
          .build()
          .await
          .unwrap();
        let changed_proxy = AsyncFancyProxy::new(&signal_conn).await.unwrap();
        let mut config_changes = changed_proxy.receive_config_changed().await.fuse();
        let mut auto_changes = changed_proxy.receive_auto_changed().await.fuse();

        {
          state.write().unwrap().config = proxy.config().await.unwrap();
          state.write().unwrap().poll_interval = proxy.poll_interval().await.unwrap();
        }

        let mut rx_stream = backend_recv.into_stream().fuse();
        loop {
          select! {
            msg = rx_stream.next() => {
              if let Some(msg) = msg {
                match msg {
                  Msg::GetPollInterval => backend_send
                    .send_async(Msg::PollInterval(proxy.poll_interval().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::GetTemps => backend_send
                    .send_async(Msg::Temps(proxy.temperatures().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::GetSpeeds => backend_send
                    .send_async(Msg::Speeds(proxy.fans_speeds().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::GetTargetSpeeds => backend_send
                    .send_async(Msg::TargetSpeeds(proxy.target_fans_speeds().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::SetTargetSpeed(i, s) => proxy.set_target_fan_speed(i, s).await.unwrap(),
                  Msg::GetCritical => backend_send
                    .send_async(Msg::Critical(proxy.critical().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::GetNames => backend_send
                    .send_async(Msg::Names(proxy.fans_names().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::GetAuto => backend_send
                    .send_async(Msg::Auto(proxy.auto().await.unwrap()))
                    .await
                    .unwrap(),
                  Msg::SetAuto(a) => proxy.set_auto(a).await.unwrap(),
                  _ => unreachable!(),
                };
              }
            },
            c = config_changes.next() => {
              if let Some(Some(c)) = c {
                let config = c.downcast_ref::<str>().unwrap().to_string();
                state.write().unwrap().config = config.clone();
                app.emit_all("config_change", config).unwrap();
              }
            },
            a = auto_changes.next() => {
              if let Some(Some(a)) = a {
                let auto = a.downcast_ref::<bool>().unwrap();
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
      get_temps,
      get_poll_interval,
      get_speeds,
      get_target_speeds,
      get_critical,
      get_config,
      get_names,
      get_auto,
      set_target_speed,
      set_auto
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
