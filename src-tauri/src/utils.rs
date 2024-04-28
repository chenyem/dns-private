use anyhow::Result;
use std::path::{Path, PathBuf};
use tauri::updater::UpdateResponse;
use tauri::{AppHandle, Manager, Wry};

pub fn app_root() -> PathBuf {
  tauri::api::path::home_dir().unwrap().join(".mdns")
}

pub fn run_check_update(app: AppHandle<Wry>, silent: bool, has_msg: Option<bool>) {
  tauri::async_runtime::spawn(async move {
    if let Ok(update_resp) = app.updater().check().await {
      if update_resp.is_update_available() {
        if silent {
          tauri::async_runtime::spawn(async move {
            silent_install(app, update_resp).await.unwrap();
          });
        } else {
          tauri::async_runtime::spawn(async move {
            prompt_for_install(app, update_resp).await.unwrap();
          });
        }
      } else if let Some(v) = has_msg {
        if v {
          tauri::api::dialog::message(
            app.app_handle().get_window("core").as_ref(),
            "mdns",
            "Your mdns is up to date",
          );
        }
      }
    }
  });
}

// Copy private api in tauri/updater/mod.rs. TODO: refactor to public api
// Prompt a dialog asking if the user want to install the new version
// Maybe we should add an option to customize it in future versions.
pub async fn prompt_for_install(app: AppHandle<Wry>, update: UpdateResponse<Wry>) -> Result<()> {
  let windows = app.windows();
  let parent_window = windows.values().next();
  let package_info = app.package_info().clone();

  let body = update.body().unwrap();
  // todo(lemarier): We should review this and make sure we have
  // something more conventional.
  let should_install = tauri::api::dialog::blocking::ask(
    parent_window,
    format!(r#"A new version of {} is available! "#, package_info.name),
    format!(
      r#"{} {} is now available -- you have {}.

Would you like to install it now?

Release Notes:
{}"#,
      package_info.name,
      update.latest_version(),
      package_info.version,
      body
    ),
  );

  if should_install {
    // Launch updater download process
    // macOS we display the `Ready to restart dialog` asking to restart
    // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
    // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
    update.download_and_install().await?;

    // Ask user if we need to restart the application
    let should_exit = tauri::api::dialog::blocking::ask(
      parent_window,
      "Ready to Restart",
      "The installation was successful, do you want to restart the application now?",
    );
    if should_exit {
      app.restart();
    }
  }

  Ok(())
}

pub async fn silent_install(app: AppHandle<Wry>, update: UpdateResponse<Wry>) -> Result<()> {
  let windows = app.windows();
  let parent_window = windows.values().next();

  // Launch updater download process
  // macOS we display the `Ready to restart dialog` asking to restart
  // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
  // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
  update.download_and_install().await?;

  // Ask user if we need to restart the application
  let should_exit = tauri::api::dialog::blocking::ask(
    parent_window,
    "Ready to Restart",
    "The silent installation was successful, do you want to restart the application now?",
  );
  if should_exit {
    app.restart();
  }

  Ok(())
}
