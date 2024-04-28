// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::AppHandle;

mod utils;
#[tauri::command]
fn greet(app: AppHandle, name: &str) -> String {
  format!(
    "Hello, {}! from Rust! and veriosn is @@ {} @@",
    name,
    app.package_info().version
  )
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let app_handle = app.handle();
      tauri::async_runtime::spawn(async { utils::run_check_update(app_handle, false, None) });
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![greet])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
