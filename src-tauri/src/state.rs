use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use bollard::Docker;
use tauri::async_runtime::JoinHandle;

use crate::models::ServerMeta;

pub struct ServerEntry {
    pub meta: ServerMeta,
    pub container_id: Option<String>,
}

pub struct AppState {
    pub docker: Option<Docker>,
    pub servers: Mutex<Vec<ServerEntry>>,
    pub log_tasks: Mutex<HashMap<String, JoinHandle<()>>>,
    pub workspace: std::path::PathBuf,
    pub config_path: std::path::PathBuf,
}

impl AppState {
    pub fn insert_log_task(&self, server_id: String, task: JoinHandle<()>) {
        self.log_tasks.lock().unwrap().insert(server_id, task);
    }
    pub fn remove_log_task(&self, server_id: &str) -> Option<JoinHandle<()>> {
        self.log_tasks.lock().unwrap().remove(server_id)
    }
    pub fn contains_log_task(&self, server_id: &str) -> bool {
        self.log_tasks.lock().unwrap().contains_key(server_id)
    }
    pub fn update_server_entry<T>(
        &self,
        server_id: &str,
        update: impl FnOnce(&mut ServerEntry) -> T,
    ) -> Option<T> {
        let mut servers = self.servers.lock().unwrap();
        servers
            .iter_mut()
            .find(|entry| entry.meta.id == server_id)
            .map(update)
    }
}

pub fn load_config(path: &Path) -> Vec<ServerMeta> {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_config(path: &Path, servers: &[ServerEntry]) {
    let metas: Vec<ServerMeta> = servers.iter().map(|e| e.meta.clone()).collect();
    if let Ok(json) = serde_json::to_string_pretty(&metas) {
        let _ = std::fs::write(path, json);
    }
}
