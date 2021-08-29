#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use zbus::Connection;

#[derive(Debug)]
struct State {
  connection: Connection,
  config: String,
  interval: usize,
}

fn main() {
  let connection = Connection::session().expect("Failed to create D-Bus connection");
  let state = State { connection };

  tauri::Builder::default()
    .manage(state)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
