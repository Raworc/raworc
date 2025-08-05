mod auth;
mod database;
mod logging;
mod models;
mod rbac;
mod rest;

use anyhow::Result;
#[cfg(unix)]
use daemonize::Daemonize;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::env;
use std::fs;
use tracing::info;
#[cfg(unix)]
use tracing::error;


fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle daemon mode first (no tokio runtime needed)
    #[cfg(unix)]
    if args.len() >= 2 && args[1] == "serve" {
        return start_server_daemon();
    }
    
    #[cfg(not(unix))]
    if args.len() >= 2 && args[1] == "serve" {
        eprintln!("Error: 'serve' command is not supported on Windows.");
        eprintln!("Please use 'raworc start' to run the server in foreground mode.");
        return Err(anyhow::anyhow!("Unsupported command on Windows"));
    }

    // For all other commands, use tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { main_async().await })
}

async fn main_async() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Default to connect if no arguments provided
    if args.len() < 2 {
        connect_to_server().await?;
        return Ok(());
    }

    match args[1].as_str() {
        "start" => {
            start_server().await?;
        }
        "stop" => {
            stop_server().await?;
        }
        "connect" => {
            connect_to_server().await?;
        }
        "auth" => {
            auth_interactive().await?;
        }
        "status" => {
            show_auth_status().await?;
        }
        "migrate" => {
            run_migrations(&args).await?;
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            println!("Unknown command: '{}'", args[1]);
            println!("Available commands are: start, stop, connect, auth, migrate, help");
            println!();
            print_help();
        }
    }

    Ok(())
}

async fn start_server() -> Result<()> {
    use std::fs;
    use std::path::Path;

    let pid_file = "/tmp/raworc.pid";

    // Check if server is already running
    if Path::new(pid_file).exists() {
        if let Ok(pid_str) = fs::read_to_string(pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is actually running
                if std::process::Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                {
                    println!("✗ Raworc server is already running (PID: {pid})");
                    println!(
                        "   Use 'raworc stop' to stop it first, or 'raworc status' to check status"
                    );
                    return Ok(());
                } else {
                    // Clean up stale PID file
                    let _ = fs::remove_file(pid_file);
                }
            }
        }
    }

    // Initialize logging for foreground mode
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_dir = current_dir.join("logs").to_string_lossy().to_string();
    if let Err(e) = logging::init_logging(&log_dir, "raworc") {
        eprintln!("Failed to initialize logging: {e}");
        return Err(e);
    }

    info!("Starting Raworc server in foreground mode...");
    rest::run_rest_server().await?;

    Ok(())
}

#[cfg(unix)]
fn start_server_daemon() -> Result<()> {
    use std::fs;
    use std::path::Path;

    let pid_file = "/tmp/raworc.pid";

    // Check if server is already running
    if Path::new(pid_file).exists() {
        if let Ok(pid_str) = fs::read_to_string(pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is actually running
                if std::process::Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
                {
                    println!("✗ Raworc server is already running (PID: {pid})");
                    println!(
                        "   Use 'raworc stop' to stop it first, or 'raworc status' to check status"
                    );
                    return Ok(());
                } else {
                    // Clean up stale PID file
                    let _ = fs::remove_file(pid_file);
                }
            }
        }
    }

    println!("Starting Raworc server in daemon mode...");

    // Initialize logging before daemonizing
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_dir = current_dir.join("logs").to_string_lossy().to_string();

    // Set up daemon configuration
    let daemonize = Daemonize::new()
        .pid_file(pid_file)
        .working_directory("/tmp")
        .umask(0o027)
        .privileged_action(|| "Entered daemon mode");

    match daemonize.start() {
        Ok(_) => {
            // We're now in the daemon process
            // The PID file is automatically created by daemonize

            // Initialize logging in the daemon process
            if let Err(e) = logging::rotate_logs_on_startup(&log_dir, "raworc") {
                eprintln!("Failed to rotate logs: {e}");
            }

            if let Err(e) = logging::init_logging(&log_dir, "raworc") {
                eprintln!("Failed to initialize logging: {e}");
                std::process::exit(1);
            }

            // Create a new tokio runtime for the daemon process
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                if let Err(e) = rest::run_rest_server().await {
                    error!("Daemon server error: {}", e);
                    std::process::exit(1);
                }
            });
        }
        Err(e) => {
            eprintln!("✗ Failed to start daemon: {e}");
            return Err(anyhow::anyhow!("Failed to start daemon: {e}"));
        }
    }

    Ok(())
}

async fn stop_server() -> Result<()> {
    let pid_file = "/tmp/raworc.pid";

    // Check if PID file exists
    if !std::path::Path::new(pid_file).exists() {
        println!("✗ Raworc server is not running");
        println!("   PID file not found at: {pid_file}");
        return Ok(());
    }

    // Read PID from file
    let pid_str = fs::read_to_string(pid_file)?;
    let pid = pid_str
        .trim()
        .parse::<u32>()
        .map_err(|e| anyhow::anyhow!("Invalid PID in file: {e}"))?;

    // Try to kill the process
    match std::process::Command::new("kill")
        .arg(pid.to_string())
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("✓ Raworc server stopped (PID: {pid})");
                // Clean up PID file
                let _ = fs::remove_file(pid_file);
            } else {
                // Process might not exist
                println!("✗ Failed to stop server: Process not found (PID: {pid})");
                // Clean up stale PID file
                let _ = fs::remove_file(pid_file);
            }
        }
        Err(e) => {
            println!("✗ Failed to stop server: {e}");
            return Err(anyhow::anyhow!("Failed to execute kill command: {e}"));
        }
    }

    Ok(())
}

async fn connect_to_server() -> Result<()> {
    println!("Connect command not yet implemented in this version.");
    println!("Please use the REST API directly.");
    Ok(())
}

async fn auth_interactive() -> Result<()> {
    println!("Auth command not yet implemented in this version.");
    println!("Please use the REST API directly with your credentials.");
    Ok(())
}

async fn show_auth_status() -> Result<()> {
    println!("Status command not yet implemented in this version.");
    println!("Please use the REST API directly.");
    Ok(())
}





fn print_help() {
    println!("USAGE:");
    println!("    raworc [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    (default)          Connect to authenticated server");
    println!("    start              Start the Raworc server in foreground");
    #[cfg(unix)]
    println!("    serve              Start the Raworc server as daemon");
    println!("    stop               Stop the running Raworc server");
    println!("    connect            Connect to authenticated server");
    println!("    auth               Authenticate with server");
    println!("    status             Show authentication status");
    println!("    migrate            Run database migrations");
    println!("    help               Show this help message");
    println!();
    println!("MIGRATE SUBCOMMANDS:");
    println!("    migrate            Run all pending migrations");
    println!("    migrate status     Show migration status");
    println!("    migrate up         Run all pending migrations");
    println!("    migrate down       Rollback last migration");
}


#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn run_migrations(args: &[String]) -> Result<()> {
    use sqlx::postgres::PgPoolOptions;
    
    // Get database URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres@localhost/raworc".to_string());
    
    // Parse subcommand
    let subcommand = if args.len() > 2 {
        args[2].as_str()
    } else {
        "up"
    };
    
    match subcommand {
        "up" | "" => {
            println!("Running database migrations...");
            println!("Database URL: {}", database_url);
            println!();
            
            // Connect to database
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await?;
            
            // Run migrations
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => {
                    println!("✓ All migrations completed successfully");
                }
                Err(e) => {
                    println!("✗ Migration failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        "status" => {
            println!("Checking migration status...");
            println!("Database URL: {}", database_url);
            println!();
            
            // Connect to database
            let pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await?;
            
            // Get migration status
            let migrations = sqlx::migrate!("./migrations");
            
            println!("Available migrations:");
            for migration in migrations.iter() {
                println!("  {} - {}", 
                    migration.version, 
                    migration.description
                );
            }
            
            // Check which migrations have been applied
            println!("\nApplied migrations:");
            match sqlx::query("SELECT version, description, installed_on FROM _sqlx_migrations ORDER BY version")
                .fetch_all(&pool)
                .await
            {
                Ok(applied) => {
                    for row in applied {
                        let version: i64 = row.get("version");
                        let description: String = row.get("description");
                        let installed_on: chrono::DateTime<chrono::Utc> = row.get("installed_on");
                        println!("  {} - {} (applied: {})", 
                            version,
                            description,
                            installed_on.format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                }
                Err(_) => {
                    println!("  No migrations table found. Run 'raworc migrate up' to initialize.");
                }
            }
        }
        "down" => {
            println!("✗ Rollback is not supported in this version");
            println!("   Please manually rollback using SQL if needed");
        }
        _ => {
            println!("Unknown migrate subcommand: {}", subcommand);
            println!();
            println!("Available subcommands:");
            println!("  up      - Run all pending migrations (default)");
            println!("  status  - Show migration status");
            println!("  down    - Rollback last migration (not supported)");
        }
    }
    
    Ok(())
}