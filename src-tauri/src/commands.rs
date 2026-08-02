use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};

use crate::compose;
use crate::docker as docker_mod;
use crate::logging;
use crate::models::{DockerStatus, ServerMeta, ServerStatus, StatusEvent};
use crate::state::{save_config, AppState, ServerEntry};
use crate::templates;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateSource {
    Template {
        port: u16,
        #[serde(rename = "memoryGb")]
        memory_gb: u32,
    },
    Yaml {
        content: String,
    },
    Existing {
        #[serde(rename = "composePath")]
        compose_path: String,
    },
}

fn generate_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos}")
}

fn sanitize_project(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    let s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "server".to_string()
    } else {
        s
    }
}

fn random_password() -> String {
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut out = String::new();
    for _ in 0..16 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        out.push(alphabet[(seed as usize) % alphabet.len()]);
    }
    out
}

fn emit_status(app: &AppHandle, server_id: &str, status: &ServerStatus) {
    let _ = app.emit(
        "server-status",
        StatusEvent {
            server_id: server_id.to_string(),
            status: status.clone(),
        },
    );
}

async fn sync_and_emit_status(app: &AppHandle, state: &AppState, server_id: &str) {
    let docker = state.docker.clone();
    let project = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .find(|e| e.meta.id == server_id)
            .map(|e| e.meta.project.clone())
    };
    let status = match (docker, project) {
        (Some(d), Some(p)) => match docker_mod::status_of_project(&d, &p).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(server_id, error = %e, "status sync failed");
                ServerStatus::Error
            }
        },
        _ => ServerStatus::Error,
    };
    {
        let mut servers = state.servers.lock().unwrap();
        if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
            entry.meta.status = status.clone();
        }
    }
    emit_status(app, server_id, &status);
}

fn find_meta(state: &AppState, server_id: &str) -> Option<ServerMeta> {
    state
        .servers
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.meta.id == server_id)
        .map(|e| e.meta.clone())
}

fn list_all_meta(state: &AppState) -> Vec<ServerMeta> {
    state
        .servers
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.meta.clone())
        .collect()
}

async fn resolve_container(state: &AppState, server_id: &str) -> Result<String, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;
    let project = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .find(|e| e.meta.id == server_id)
            .map(|e| e.meta.project.clone())
            .ok_or("Server not found".to_string())?
    };
    let stored = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .find(|e| e.meta.id == server_id)
            .and_then(|e| e.container_id.clone())
    };
    if let Some(id) = stored {
        return Ok(id);
    }
    let found = docker_mod::find_container_id(&docker, &project).await?;
    if let Some(id) = found {
        {
            let mut servers = state.servers.lock().unwrap();
            if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
                entry.container_id = Some(id.clone());
            }
        }
        Ok(id)
    } else {
        Err("Server is not running".to_string())
    }
}

#[tauri::command]
pub fn list_servers(state: State<'_, AppState>) -> Vec<ServerMeta> {
    list_all_meta(&state)
}

#[tauri::command]
pub fn get_template_info() -> String {
    templates::TEMPLATE_NAME.to_string()
}

#[tauri::command]
pub async fn docker_status(state: State<'_, AppState>) -> Result<DockerStatus, String> {
    let compose_ok = compose::compose_available();
    let docker = match state.docker.clone() {
        Some(d) => d,
        None => {
            return Ok(DockerStatus {
                engine_ok: false,
                compose_ok,
                error: Some("Docker client unavailable".to_string()),
            })
        }
    };
    match docker.ping().await {
        Ok(_) => Ok(DockerStatus {
            engine_ok: true,
            compose_ok,
            error: None,
        }),
        Err(e) => Ok(DockerStatus {
            engine_ok: false,
            compose_ok,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn create_server(
    state: State<'_, AppState>,
    name: String,
    source: CreateSource,
) -> Result<ServerMeta, String> {
    let project = sanitize_project(&name);
    if project.is_empty() {
        return Err("Invalid server name".into());
    }
    let (server_dir, compose_path) = {
        if let CreateSource::Existing { compose_path } = &source {
            let path = PathBuf::from(compose_path);
            if let Some(parent) = path.parent() {
                (parent.to_path_buf(), path)
            } else {
                return Err(format!("Compose file in the root directory!"));
            }
        } else {
            let dir = state.workspace.join(&project);
            let path = dir.join("docker_compose.yml");
            if path.exists() {
                return Err(format!("Server '{name}' already exists"));
            }
            (dir, path)
        }
    };

    let content = match &source {
        CreateSource::Template { port, memory_gb } => {
            templates::render_template(&project, *port, *memory_gb, &random_password())
        }
        CreateSource::Yaml { content } => content.clone(),
        CreateSource::Existing { compose_path } => {
            fs::read_to_string(compose_path).map_err(|e| format!("Failed to read file: {e}"))?
        }
    };

    serde_yaml::from_str::<serde_yaml::Value>(&content)
        .map_err(|e| format!("Invalid compose YAML: {e}"))?;

    if !matches!(source, CreateSource::Existing { compose_path: _ }) {
        fs::create_dir_all(&server_dir).map_err(|e| e.to_string())?;
        fs::write(&compose_path, &content).map_err(|e| e.to_string())?;
    }

    let has_rcon = templates::detect_rcon(&content);
    let meta = ServerMeta {
        id: generate_id(),
        name: name.clone(),
        project: project.clone(),
        path: server_dir.to_string_lossy().to_string(),
        status: ServerStatus::Stopped,
        has_rcon,
    };

    {
        let mut servers = state.servers.lock().unwrap();
        servers.push(ServerEntry {
            meta: meta.clone(),
            container_id: None,
        });
        save_config(&state.config_path, &servers);
    }

    tracing::info!(name, project, has_rcon, "server created");
    Ok(meta)
}

#[tauri::command]
pub async fn start_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerMeta, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;

    let (project, path) = {
        let mut servers = state.servers.lock().unwrap();
        let entry = servers
            .iter_mut()
            .find(|e| e.meta.id == server_id)
            .ok_or("Server not found".to_string())?;
        entry.meta.status = ServerStatus::Starting;
        (entry.meta.project.clone(), PathBuf::from(&entry.meta.path))
    };
    tracing::info!(server_id, project, "start_server: setting status Starting");
    emit_status(&app, &server_id, &ServerStatus::Starting);

    let result = async {
        let compose_file = path.join("docker-compose.yml");
        let project_up = project.clone();
        tracing::debug!(project, compose_file = %compose_file.display(), "start_server: running compose up");
        tauri::async_runtime::spawn_blocking(move || compose::compose_up(&project_up, &compose_file))
            .await
            .map_err(|e| e.to_string())??;

        let container_id = docker_mod::find_container_id(&docker, &project).await?;

        {
            let mut servers = state.servers.lock().unwrap();
            if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
                entry.meta.status = ServerStatus::Running;
                entry.container_id = container_id.clone();
            }
        }
        tracing::info!(server_id, project, container_id = container_id.as_deref().map(|s| &s[..s.len().min(12)]), "start_server: running");
        emit_status(&app, &server_id, &ServerStatus::Running);

        if let Some(cid) = container_id {
            let handle = tauri::async_runtime::spawn(logging::stream_logs(
                app.clone(),
                docker,
                server_id.clone(),
                cid,
            ));
            state.log_tasks.lock().unwrap().insert(server_id.clone(), handle);
        }

        find_meta(&state, &server_id).ok_or("Server not found".to_string())
    }
    .await;

    if let Err(e) = &result {
        tracing::error!(server_id, project, error = %e, "start_server failed");
        sync_and_emit_status(&app, &state, &server_id).await;
    }
    result
}

#[tauri::command]
pub async fn stop_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerMeta, String> {
    if let Some(handle) = state.log_tasks.lock().unwrap().remove(&server_id) {
        handle.abort();
    }

    let (project, path) = {
        let mut servers = state.servers.lock().unwrap();
        let entry = servers
            .iter_mut()
            .find(|e| e.meta.id == server_id)
            .ok_or("Server not found".to_string())?;
        entry.meta.status = ServerStatus::Stopping;
        (entry.meta.project.clone(), PathBuf::from(&entry.meta.path))
    };
    tracing::info!(server_id, project, "stop_server: setting status Stopping");
    emit_status(&app, &server_id, &ServerStatus::Stopping);

    let result = async {
        let compose_file = path.join("docker-compose.yml");
        let project_stop = project.clone();
        tracing::debug!(project, "stop_server: running compose stop");
        tauri::async_runtime::spawn_blocking(move || {
            compose::compose_stop(&project_stop, &compose_file)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

        {
            let mut servers = state.servers.lock().unwrap();
            if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
                entry.meta.status = ServerStatus::Stopped;
                entry.container_id = None;
            }
        }
        emit_status(&app, &server_id, &ServerStatus::Stopped);

        find_meta(&state, &server_id).ok_or("Server not found".to_string())
    }
    .await;

    if let Err(e) = &result {
        tracing::error!(server_id, project, error = %e, "stop_server failed");
        sync_and_emit_status(&app, &state, &server_id).await;
    }
    result
}

#[tauri::command]
pub async fn restart_server(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerMeta, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;
    if let Some(handle) = state.log_tasks.lock().unwrap().remove(&server_id) {
        handle.abort();
    }

    let (project, path) = {
        let mut servers = state.servers.lock().unwrap();
        let entry = servers
            .iter_mut()
            .find(|e| e.meta.id == server_id)
            .ok_or("Server not found".to_string())?;
        (entry.meta.project.clone(), PathBuf::from(&entry.meta.path))
    };
    tracing::info!(
        server_id,
        project,
        "restart_server: running compose restart"
    );

    let result = async {
        let compose_file = path.join("docker-compose.yml");
        let project_restart = project.clone();
        tauri::async_runtime::spawn_blocking(move || {
            compose::compose_restart(&project_restart, &compose_file)
        })
        .await
        .map_err(|e| e.to_string())??;

        let container_id = docker_mod::find_container_id(&docker, &project).await?;

        {
            let mut servers = state.servers.lock().unwrap();
            if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
                entry.meta.status = ServerStatus::Running;
                entry.container_id = container_id.clone();
            }
        }
        emit_status(&app, &server_id, &ServerStatus::Running);

        if let Some(cid) = container_id {
            let handle = tauri::async_runtime::spawn(logging::stream_logs(
                app.clone(),
                docker,
                server_id.clone(),
                cid,
            ));
            state
                .log_tasks
                .lock()
                .unwrap()
                .insert(server_id.clone(), handle);
        }

        find_meta(&state, &server_id).ok_or("Server not found".to_string())
    }
    .await;

    if let Err(e) = &result {
        tracing::error!(server_id, project, error = %e, "restart_server failed");
        sync_and_emit_status(&app, &state, &server_id).await;
    }
    result
}

#[tauri::command]
pub async fn remove_server(state: State<'_, AppState>, server_id: String) -> Result<(), String> {
    if let Some(handle) = state.log_tasks.lock().unwrap().remove(&server_id) {
        handle.abort();
    }

    let (project, path) = {
        let mut servers = state.servers.lock().unwrap();
        let idx = servers
            .iter()
            .position(|e| e.meta.id == server_id)
            .ok_or("Server not found".to_string())?;
        let entry = servers.remove(idx);
        save_config(&state.config_path, &servers);
        (entry.meta.project, PathBuf::from(&entry.meta.path))
    };

    let compose_file = path.join("docker-compose.yml");
    if compose_file.exists() {
        let _ = tauri::async_runtime::spawn_blocking(move || {
            compose::compose_down(&project, &compose_file)
        })
        .await;
    }
    let _ = fs::remove_dir_all(&path);
    tracing::info!(server_id, "server removed");
    Ok(())
}

#[tauri::command]
pub async fn get_server_status(
    app: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
) -> Result<ServerMeta, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;
    let project = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .find(|e| e.meta.id == server_id)
            .map(|e| e.meta.project.clone())
            .ok_or("Server not found".to_string())?
    };
    let status = docker_mod::status_of_project(&docker, &project).await?;
    {
        let mut servers = state.servers.lock().unwrap();
        if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == server_id) {
            entry.meta.status = status.clone();
        }
    }
    emit_status(&app, &server_id, &status);
    find_meta(&state, &server_id).ok_or("Server not found".to_string())
}

#[tauri::command]
pub async fn refresh_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ServerMeta>, String> {
    let docker = match state.docker.clone() {
        Some(d) => d,
        None => return Err("Docker unavailable".into()),
    };
    let projects: Vec<(String, String)> = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .map(|e| (e.meta.id.clone(), e.meta.project.clone()))
            .collect()
    };
    for (id, project) in projects {
        if let Ok(status) = docker_mod::status_of_project(&docker, &project).await {
            {
                let mut servers = state.servers.lock().unwrap();
                if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == id) {
                    entry.meta.status = status.clone();
                }
            }
            emit_status(&app, &id, &status);
            if status == ServerStatus::Running {
                let already = state.log_tasks.lock().unwrap().contains_key(&id);
                if !already {
                    if let Ok(Some(cid)) = docker_mod::find_container_id(&docker, &project).await {
                        {
                            let mut servers = state.servers.lock().unwrap();
                            if let Some(entry) = servers.iter_mut().find(|e| e.meta.id == id) {
                                entry.container_id = Some(cid.clone());
                            }
                        }
                        let handle = tauri::async_runtime::spawn(logging::stream_logs(
                            app.clone(),
                            docker.clone(),
                            id.clone(),
                            cid,
                        ));
                        state.log_tasks.lock().unwrap().insert(id.clone(), handle);
                    }
                }
            }
        }
    }
    let metas = list_all_meta(&state);
    tracing::debug!(count = metas.len(), "refresh_status done");
    Ok(metas)
}

#[tauri::command]
pub async fn send_command(
    state: State<'_, AppState>,
    server_id: String,
    command: String,
) -> Result<String, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;
    let container_id = resolve_container(&state, &server_id).await?;
    let has_rcon = {
        let servers = state.servers.lock().unwrap();
        servers
            .iter()
            .find(|e| e.meta.id == server_id)
            .map(|e| e.meta.has_rcon)
            .unwrap_or(false)
    };
    let cmd = command.trim().to_string();
    if cmd.is_empty() {
        return Ok(String::new());
    }

    if has_rcon {
        match docker_mod::run_exec(&docker, &container_id, &["rcon-cli", cmd.as_str()]).await {
            Ok(out) => {
                tracing::debug!(server_id, cmd = %cmd, "command sent via rcon");
                return Ok(out);
            }
            Err(e) => {
                tracing::warn!(
                    server_id,
                    cmd = %cmd,
                    error = %e,
                    "rcon failed, falling back to container stdin"
                );
            }
        }
    }

    match docker_mod::send_stdin(&docker, &container_id, &cmd).await {
        Ok(out) => {
            tracing::debug!(server_id, cmd = %cmd, "command sent via container stdin");
            Ok(out)
        }
        Err(e) => {
            tracing::error!(server_id, cmd = %cmd, error = %e, "send_command failed");
            Err(format!("Failed to send command: {e}"))
        }
    }
}

#[tauri::command]
pub async fn get_server_logs(
    state: State<'_, AppState>,
    server_id: String,
    lines: Option<u32>,
) -> Result<Vec<String>, String> {
    let docker = state
        .docker
        .clone()
        .ok_or("Docker unavailable".to_string())?;
    let container_id = resolve_container(&state, &server_id).await?;
    let tail = lines.unwrap_or(200).to_string();
    logging::recent_logs(&docker, &container_id, &tail).await
}
