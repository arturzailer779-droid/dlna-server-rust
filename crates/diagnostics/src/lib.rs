use anyhow::{Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::Mutex,
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn default_log_path() -> Result<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("dlna-video-server-2").join("server.log"));
    }
    let home = std::env::var_os("HOME").context("Не найдена переменная HOME")?;
    Ok(PathBuf::from(home).join(".cache").join("dlna-video-server-2").join("server.log"))
}

pub fn init_logging() -> Result<PathBuf> {
    let log_path = default_log_path()?;
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("Не удалось открыть файл журнала {}", log_path.display()))?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .with(fmt::layer().with_ansi(false).with_writer(Mutex::new(file)))
        .try_init()
        .ok();

    Ok(log_path)
}

pub fn append_event(message: &str) {
    tracing::info!("{message}");
    if let Ok(path) = default_log_path()
        && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "{message}");
        }
}
