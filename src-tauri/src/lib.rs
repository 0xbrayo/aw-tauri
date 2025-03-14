use aw_server::endpoints::build_rocket;
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Condvar, Mutex, OnceLock};

use log::info;
use tauri::{
    AppHandle, Manager,
};

pub struct AppHandleWrapper(Mutex<AppHandle>);

impl Drop for AppHandleWrapper {
    fn drop(&mut self) {
        let (_lock, cvar) = &*HANDLE_CONDVAR;
        cvar.notify_all();
    }
}

// static HANDLE: OnceLock<AppHandleWrapper> = OnceLock::new();
lazy_static! {
    static ref HANDLE_CONDVAR: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
}


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            {
                let testing = true;
                let legacy_import = false;

                let mut aw_config = aw_server::config::create_config(testing);
                aw_config.port = 5600;
                let db_path = aw_server::dirs::db_path(testing)
                    .expect("Failed to get db path")
                    .to_str()
                    .unwrap()
                    .to_string();
                let device_id = aw_server::device_id::get_device_id();

                let webui_var = std::env::var("AW_WEBUI_DIR");

                let asset_path_opt = if let Ok(path_str) = &webui_var {
                    let asset_path = PathBuf::from(&path_str);
                    if asset_path.exists() {
                        info!("Using webui path: {}", path_str);
                        Some(asset_path)
                    } else {
                        panic!("Path set via env var AW_WEBUI_DIR does not exist");
                    }
                } else {
                    println!("Using bundled assets");
                    None
                };

                let server_state = aw_server::endpoints::ServerState {
                    // Even if legacy_import is set to true it is disabled on Android so
                    // it will not happen there
                    datastore: Mutex::new(aw_datastore::Datastore::new(db_path, legacy_import)),
                    asset_resolver: aw_server::endpoints::AssetResolver::new(asset_path_opt),
                    device_id,
                };
                
                tauri::async_runtime::spawn(build_rocket(server_state, aw_config).launch());
                let url = format!("http://localhost:{}/", 5600);
                let mut main_window = app.get_webview_window("main").unwrap();

                main_window
                           .eval(&format!("window.location.href = '{}'", url))
                           .map_err(|e| e.to_string())?;

            }
            
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
