use bollard::container::LogOutput;
use bollard::query_parameters::LogsOptionsBuilder;
use bollard::Docker;
use futures_util::StreamExt;
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter};

use crate::models::LogEvent;

pub async fn stream_logs(app: AppHandle, docker: Docker, server_id: String, container_id: String) {
    tracing::info!(server_id, container_id = %&container_id[..container_id.len().min(12)], "log stream started");
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(true)
        .tail("200")
        .build();
    let stream = docker.logs(&container_id, Some(options));
    futures_util::pin_mut!(stream);
    while let Some(item) = stream.next().await {
        match item {
            Ok(LogOutput::StdOut { message })
            | Ok(LogOutput::StdErr { message })
            | Ok(LogOutput::Console { message })
            | Ok(LogOutput::StdIn { message }) => {
                let text = String::from_utf8_lossy(&message);
                for line in text.lines() {
                    let line = line.trim_end_matches('\r').to_string();
                    if !line.is_empty() {
                        let _ = app.emit(
                            "server-log",
                            LogEvent {
                                server_id: server_id.clone(),
                                line,
                            },
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(server_id, error = %e, "log stream error, ending");
                break;
            }
        }
    }
    tracing::info!(server_id, "log stream ended");
}

pub async fn recent_logs(
    docker: &Docker,
    container_id: &str,
    lines: &str,
) -> Result<Vec<String>, String> {
    let options = LogsOptionsBuilder::default()
        .stdout(true)
        .stderr(true)
        .follow(false)
        .tail(lines)
        .build();
    let stream = docker.logs(container_id, Some(options));
    futures_util::pin_mut!(stream);
    let mut out = Vec::new();
    while let Some(item) = stream.next().await {
        match item {
            Ok(LogOutput::StdOut { message })
            | Ok(LogOutput::StdErr { message })
            | Ok(LogOutput::Console { message }) => {
                let text = String::from_utf8_lossy(&message);
                for line in text.lines() {
                    out.push(line.trim_end_matches('\r').to_string());
                }
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "recent_logs stream error");
                break;
            }
        }
    }
    tracing::debug!(lines = out.len(), "recent_logs fetched");
    Ok(out)
}

pub fn start_logging_task(
    app: AppHandle,
    docker: Docker,
    server_id: String,
    container_id: String,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(stream_logs(app, docker, server_id, container_id))
}
