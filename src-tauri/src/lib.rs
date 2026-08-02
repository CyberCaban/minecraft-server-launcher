mod commands;
mod compose;
mod docker;
mod logging;
mod models;
mod state;
mod templates;
mod tracing_setup;

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::Manager;

use state::{load_config, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            tracing_setup::init(&app_data.join("logs"));
            tracing::info!(?app_data, "app starting");
            let workspace = app_data.join("servers");
            std::fs::create_dir_all(&workspace)?;
            let config_path = app_data.join("servers.json");
            let servers: Vec<state::ServerEntry> = load_config(&config_path)
                .into_iter()
                .map(|meta| state::ServerEntry {
                    meta,
                    container_id: None,
                })
                .collect();
            tracing::info!(count = servers.len(), ?config_path, "config loaded");
            let docker = docker::connect().ok();
            if docker.is_none() {
                tracing::error!("docker client unavailable");
            }
            app.manage(AppState {
                docker,
                servers: Mutex::new(servers),
                log_tasks: Mutex::new(HashMap::new()),
                workspace,
                config_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::docker_status,
            commands::list_servers,
            commands::get_template_info,
            commands::create_server,
            commands::remove_server,
            commands::start_server,
            commands::stop_server,
            commands::restart_server,
            commands::get_server_status,
            commands::refresh_status,
            commands::send_command,
            commands::get_server_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
