use std::collections::HashMap;

use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::models::ContainerSummaryStateEnum;
use bollard::query_parameters::{AttachContainerOptionsBuilder, ListContainersOptionsBuilder};
use bollard::Docker;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::models::ServerStatus;

pub fn connect() -> Result<Docker, String> {
    match Docker::connect_with_local_defaults() {
        Ok(d) => {
            tracing::info!("docker client connected");
            Ok(d)
        }
        Err(e) => {
            tracing::error!(error = %e, "docker connection failed");
            Err(format!("Docker connection failed: {e}"))
        }
    }
}

fn project_filters(project: &str) -> HashMap<String, Vec<String>> {
    let mut filters = HashMap::new();
    filters.insert(
        "label".to_string(),
        vec![format!("com.docker.compose.project={project}")],
    );
    filters
}

pub async fn find_container_id(
    docker: &Docker,
    project: &str,
) -> Result<Option<String>, String> {
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&project_filters(project))
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| {
            tracing::error!(project, error = %e, "list_containers failed");
            e.to_string()
        })?;
    let id = containers.into_iter().filter_map(|c| c.id).next();
    tracing::debug!(project, found = id.is_some(), "find_container_id");
    Ok(id)
}

pub async fn status_of_project(docker: &Docker, project: &str) -> Result<ServerStatus, String> {
    let options = ListContainersOptionsBuilder::default()
        .all(true)
        .filters(&project_filters(project))
        .build();
    let containers = docker
        .list_containers(Some(options))
        .await
        .map_err(|e| {
            tracing::error!(project, error = %e, "status list_containers failed");
            e.to_string()
        })?;
    if containers.is_empty() {
        tracing::debug!(project, "no containers found, status=Stopped");
        return Ok(ServerStatus::Stopped);
    }
    let any_running = containers
        .iter()
        .any(|c| c.state == Some(ContainerSummaryStateEnum::RUNNING));
    let status = if any_running {
        ServerStatus::Running
    } else {
        ServerStatus::Stopped
    };
    tracing::debug!(project, status = ?status, "status_of_project");
    Ok(status)
}

pub async fn run_exec(
    docker: &Docker,
    container_id: &str,
    args: &[&str],
) -> Result<String, String> {
    let cmd: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let config = CreateExecOptions::<String> {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(cmd),
        ..Default::default()
    };
    let created = docker
        .create_exec(container_id, config)
        .await
        .map_err(|e| {
            tracing::error!(cmd = ?args, error = %e, "create_exec failed");
            e.to_string()
        })?;
    let result = docker
        .start_exec(&created.id, None)
        .await
        .map_err(|e| {
            tracing::error!(cmd = ?args, error = %e, "start_exec failed");
            e.to_string()
        })?;

    match result {
        StartExecResults::Detached => {
            tracing::debug!(cmd = ?args, "exec detached");
            Ok(String::new())
        }
        StartExecResults::Attached { output, .. } => {
            let mut text = String::new();
            futures_util::pin_mut!(output);
            while let Some(item) = output.next().await {
                match item {
                    Ok(LogOutput::StdOut { message })
                    | Ok(LogOutput::StdErr { message })
                    | Ok(LogOutput::Console { message }) => {
                        text.push_str(&String::from_utf8_lossy(&message));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!(error = %e, "exec stream error");
                        break;
                    }
                }
            }
            tracing::debug!(cmd = ?args, len = text.len(), "exec completed");
            Ok(text)
        }
    }
}

async fn container_is_tty(docker: &Docker, container_id: &str) -> Result<bool, String> {
    let info = docker
        .inspect_container(container_id, None)
        .await
        .map_err(|e| format!("inspect failed: {e}"))?;
    Ok(info.config.and_then(|c| c.tty).unwrap_or(false))
}

async fn send_via_attach(
    docker: &Docker,
    container_id: &str,
    command: &str,
) -> Result<String, String> {
    let options = AttachContainerOptionsBuilder::default()
        .stdin(true)
        .stream(true)
        .build();
    let mut attach = docker
        .attach_container(container_id, Some(options))
        .await
        .map_err(|e| format!("attach failed: {e}"))?;
    let mut payload = command.trim_end().to_string();
    payload.push('\n');
    attach
        .input
        .write_all(payload.as_bytes())
        .await
        .map_err(|e| format!("write to stdin failed: {e}"))?;
    attach
        .input
        .flush()
        .await
        .map_err(|e| format!("flush failed: {e}"))?;
    tracing::debug!(command, "command written via attach");
    Ok(format!("> {command}"))
}

async fn send_via_proc_fd(
    docker: &Docker,
    container_id: &str,
    command: &str,
) -> Result<String, String> {
    let escaped = command.replace('\'', "'\\''");
    let shell = format!("printf '%s\\n' '{escaped}' > /proc/1/fd/0");
    let config = CreateExecOptions::<String> {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(vec!["sh".to_string(), "-c".to_string(), shell]),
        ..Default::default()
    };
    let created = docker
        .create_exec(container_id, config)
        .await
        .map_err(|e| format!("create_exec failed: {e}"))?;
    docker
        .start_exec(&created.id, None)
        .await
        .map_err(|e| format!("start_exec failed: {e}"))?;
    tracing::debug!(command, "command written via /proc/1/fd/0");
    Ok(format!("> {command}"))
}

pub async fn send_stdin(
    docker: &Docker,
    container_id: &str,
    command: &str,
) -> Result<String, String> {
    match container_is_tty(docker, container_id).await {
        Ok(true) => send_via_attach(docker, container_id, command).await,
        Ok(false) => send_via_proc_fd(docker, container_id, command).await,
        Err(e) => {
            tracing::warn!(error = %e, "tty detection failed, using attach");
            send_via_attach(docker, container_id, command).await
        }
    }
}
