use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;

pub fn init(log_dir: &Path) {
    let _ = std::fs::create_dir_all(log_dir);
    let log_file = log_dir.join("server-launcher.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .ok();

    let default = "info,server_launcher_lib=debug,hyper=warn,tao=warn,wry=warn".to_string();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();
    if let Some(f) = file {
        layers.push(
            tracing_subscriber::fmt::Layer::new()
                .with_ansi(false)
                .with_target(false)
                .with_writer(Arc::new(f))
                .with_filter(filter.clone())
                .boxed(),
        );
    }
    layers.push(
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(false)
            .with_target(false)
            .with_writer(std::io::stdout)
            .with_filter(filter)
            .boxed(),
    );

    let _ = tracing_subscriber::registry().with(layers).init();
    let _ = tracing_log::LogTracer::init();
}
