use std::path::Path;
use tracing::info;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging(log_dir: &str, service_name: &str) -> Result<(), anyhow::Error> {
    // Rotate logs on startup
    let _ = rotate_logs_on_startup(log_dir, service_name);
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(log_dir)?;

    // Set up file appender with daily rotation
    let file_appender = rolling::daily(log_dir, format!("{service_name}.log"));
    let (non_blocking_file, _guard) = non_blocking(file_appender);

    // Set up console output
    let (non_blocking_stdout, _guard2) = non_blocking(std::io::stdout());

    // Create layers
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_ansi(false) // No colors in file logs
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    let console_layer = fmt::layer()
        .with_writer(non_blocking_stdout)
        .with_ansi(true) // Colors for console
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false);

    // Set up environment filter (can be controlled via RUST_LOG env var)
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    // Forget the guards to keep them alive for the entire program duration
    std::mem::forget(_guard);
    std::mem::forget(_guard2);

    info!("Logging initialized - logs will be written to {log_dir}/{service_name}.log");

    Ok(())
}

pub fn rotate_logs_on_startup(log_dir: &str, service_name: &str) -> Result<(), anyhow::Error> {
    let log_file = format!("{log_dir}/{service_name}.log");
    let log_path = Path::new(&log_file);

    if log_path.exists() {
        // Create backup with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = format!("{log_dir}/{service_name}.{timestamp}.log");

        std::fs::rename(&log_file, &backup_file)?;
        info!("Previous log file backed up to: {backup_file}");
    }

    Ok(())
}
