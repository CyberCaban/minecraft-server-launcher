use std::path::Path;
use std::time::Instant;

fn run_compose(args: &[String]) -> Result<(), String> {
    let start = Instant::now();
    let args_display: Vec<&str> = args.iter().map(String::as_str).collect();
    tracing::debug!(args = ?args_display, "docker compose starting");
    let output = std::process::Command::new("docker")
        .arg("compose")
        .args(args)
        .output()
        .map_err(|e| {
            tracing::error!(args = ?args_display, error = %e, "failed to launch docker");
            format!("Failed to launch docker: {e}")
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success() {
        tracing::info!(
            args = ?args_display,
            elapsed_ms = start.elapsed().as_millis(),
            "docker compose ok"
        );
        let tail: Vec<&str> = stderr.lines().rev().take(5).collect();
        if !tail.is_empty() {
            tracing::debug!(args = ?args_display, output = ?tail, "docker compose stderr tail");
        }
        return Ok(());
    }
    let combined = format!("{stdout}\n{stderr}");
    let stdout_trim = stdout.trim();
    let stderr_trim = stderr.trim();
    tracing::error!(
        args = ?args_display,
        elapsed_ms = start.elapsed().as_millis(),
        stdout = %stdout_trim,
        stderr = %stderr_trim,
        "docker compose failed"
    );
    Err(combined.trim().to_string())
}

fn base_args(project: &str, file: &Path) -> Vec<String> {
    vec![
        "-p".to_string(),
        project.to_string(),
        "-f".to_string(),
        file.to_string_lossy().to_string(),
    ]
}

pub fn compose_up(project: &str, file: &Path) -> Result<(), String> {
    let mut args = base_args(project, file);
    args.extend(["up".to_string(), "-d".to_string()]);
    run_compose(&args)
}

pub fn compose_stop(project: &str, file: &Path) -> Result<(), String> {
    let mut args = base_args(project, file);
    args.push("stop".to_string());
    run_compose(&args)
}

pub fn compose_restart(project: &str, file: &Path) -> Result<(), String> {
    let mut args = base_args(project, file);
    args.push("restart".to_string());
    run_compose(&args)
}

pub fn compose_down(project: &str, file: &Path) -> Result<(), String> {
    let mut args = base_args(project, file);
    args.extend(["down".to_string(), "--remove-orphans".to_string()]);
    run_compose(&args)
}

pub fn compose_available() -> bool {
    let ok = std::process::Command::new("docker")
        .args(["compose", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    tracing::debug!(available = ok, "docker compose availability checked");
    ok
}
