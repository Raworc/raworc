mod auth;
mod database;
mod logging;
mod models;
mod rbac;
mod rest;
mod docker;

use anyhow::Result;
#[cfg(unix)]
use daemonize::Daemonize;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper, Result as RustylineResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing::info;
#[cfg(unix)]
use tracing::error;

#[derive(Helper)]
struct RaworcHelper;

impl Completer for RaworcHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let commands = ["/help", "/status", "/quit", "/q", "/api"];

        if line.is_empty() || line.starts_with('/') {
            let matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            Ok((0, matches))
        } else {
            // For non-command input, check if they're typing common commands
            let simple_commands = ["help", "status", "quit", "q", "exit"];
            let matches: Vec<Pair> = simple_commands
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            Ok((0, matches))
        }
    }
}

impl Hinter for RaworcHelper {
    type Hint = String;
}

impl Highlighter for RaworcHelper {}

impl Validator for RaworcHelper {}

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
        "help" | "--help" | "-h" => {
            print_help();
        }
        _ => {
            println!("Unknown command: '{}'", args[1]);
            println!("Available commands are: start, stop, connect, auth, help");
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
            println!("✗ Failed to daemonize: {e}");
            println!("Check log files in: {log_dir}");
            return Err(anyhow::anyhow!("Daemon startup failed: {}", e));
        }
    }

    Ok(())
}

async fn stop_server() -> Result<()> {
    use std::fs;
    use std::process;

    let pid_file = "/tmp/raworc.pid";

    match fs::read_to_string(pid_file) {
        Ok(pid_str) => {
            match pid_str.trim().parse::<u32>() {
                Ok(pid) => {
                    println!("Stopping Raworc server (PID: {pid})...");

                    // Try to kill the process
                    match process::Command::new("kill").arg(pid.to_string()).output() {
                        Ok(output) => {
                            if output.status.success() {
                                println!("Server stopped successfully.");
                                // Remove PID file
                                let _ = fs::remove_file(pid_file);
                            } else {
                                eprintln!(
                                    "Failed to stop server: {stderr}",
                                    stderr = String::from_utf8_lossy(&output.stderr)
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to execute kill command: {e}");
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Invalid PID in {pid_file}");
                    let _ = fs::remove_file(pid_file);
                }
            }
        }
        Err(_) => {
            println!("No running Raworc server found (no PID file).");
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthConfig {
    server: String,
    token: String,
}

// Token and config management
fn get_raworc_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".raworc"))
}

fn get_config_file() -> Result<PathBuf> {
    let raworc_dir = get_raworc_dir()?;
    Ok(raworc_dir.join("auth.yaml"))
}

async fn store_auth_config(server_url: &str, token: &str) -> Result<()> {
    let raworc_dir = get_raworc_dir()?;
    fs::create_dir_all(&raworc_dir)?;

    let config = AuthConfig {
        server: server_url.to_string(),
        token: token.to_string(),
    };

    let config_file = get_config_file()?;
    let yaml_content = serde_yaml::to_string(&config)?;
    fs::write(config_file, yaml_content)?;

    Ok(())
}

fn load_auth_config() -> Result<Option<AuthConfig>> {
    let config_file = get_config_file()?;
    match fs::read_to_string(config_file) {
        Ok(content) => match serde_yaml::from_str::<AuthConfig>(&content) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None),
        },
        Err(_) => Ok(None),
    }
}

async fn validate_token(server_url: &str, token: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    match client
        .get(format!("{server_url}/api/v0/auth/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(result) = response.json::<Value>().await {
                    if let Some(user) = result.get("user").and_then(|u| u.as_str()) {
                        return Ok(Some(user.to_string()));
                    }
                }
            }
            Ok(None)
        }
        Err(_) => Ok(None),
    }
}

async fn auth_interactive() -> Result<()> {
    println!("Raworc Authentication");
    println!();
    println!("Choose authentication method:");
    println!("1. Login with service account");
    println!("2. Store JWT token directly");
    println!();
    print!("Enter choice (1 or 2): ");
    io::stdout().flush()?;

    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice = choice.trim();

    match choice {
        "1" => auth_login().await?,
        "2" => auth_token_interactive().await?,
        _ => {
            println!("Invalid choice. Please enter 1 or 2.");
            return Ok(());
        }
    }

    Ok(())
}

async fn auth_login() -> Result<()> {
    println!("Service Account Login");
    print!("Server URL: ");
    io::stdout().flush()?;
    let mut server_url = String::new();
    io::stdin().read_line(&mut server_url)?;
    let server_url = server_url.trim();

    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();

    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;

    println!("Authenticating...");

    let client = reqwest::Client::new();
    let login_request = json!({
        "user": username,
        "pass": password
    });

    let response = client
        .post(format!("{server_url}/api/v0/auth/internal"))
        .header("Content-Type", "application/json")
        .json(&login_request)
        .send()
        .await?;

    if response.status().is_success() {
        let result: Value = response.json().await?;
        if let Some(token) = result.get("token").and_then(|t| t.as_str()) {
            store_auth_config(server_url, token).await?;
            if let Some(user) = validate_token(server_url, token).await? {
                println!();
                println!("✓ Authentication successful!");
                println!("   User: {user}");
                println!("   Server: {server_url}");
                println!();
                println!("You can now use 'raworc' to connect to this server.");
            }
            return Ok(());
        } else {
            println!("✗ Authentication failed: Invalid response");
        }
    } else {
        println!(
            "✗ Authentication failed: Server returned {status}",
            status = response.status()
        );
    }
    Ok(())
}

async fn auth_token_interactive() -> Result<()> {
    print!("Server URL: ");
    io::stdout().flush()?;
    let mut server_url = String::new();
    io::stdin().read_line(&mut server_url)?;
    let server_url = server_url.trim();

    print!("JWT Token: ");
    io::stdout().flush()?;
    let token = rpassword::read_password()?;

    println!("Validating token...");
    if let Some(user) = validate_token(server_url, &token).await? {
        store_auth_config(server_url, &token).await?;
        println!();
        println!("✓ Authentication successful!");
        println!("   User: {user}");
        println!("   Server: {server_url}");
        println!();
        println!("You can now use 'raworc' to connect to this server.");
    } else {
        println!("✗ Invalid token or server unreachable");
    }
    Ok(())
}

async fn get_auth_status() -> Result<String> {
    // Check if auth config exists
    match load_auth_config()? {
        Some(config) => {
            // Check server reachability using REST endpoint
            let client = reqwest::Client::new();
            let server_reachable = match client
                .get(format!("{}/api/v0/version", config.server))
                .send()
                .await
            {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            };

            if server_reachable {
                // Server is reachable, check if token is valid
                if let Some(user) = validate_token(&config.server, &config.token).await? {
                    Ok(format!(
                        "  ✓ Logged in as: {user} ({server})",
                        server = config.server
                    ))
                } else {
                    Ok(format!("  ✗ Not valid ({server})", server = config.server))
                }
            } else {
                Ok(format!(
                    "  ✗ Not reachable ({server})",
                    server = config.server
                ))
            }
        }
        None => Ok("  ✗ Not authenticated".to_string()),
    }
}

async fn show_auth_status() -> Result<()> {
    let status = get_auth_status().await?;
    println!("Authentication Status:");
    println!("{status}");

    if status.contains("Not valid") {
        println!();
        println!("Run 'raworc auth' to re-authenticate.");
    } else if status.contains("Not reachable") {
        println!();
        println!(
            "Check server status or run 'raworc auth' to authenticate with a different server."
        );
    } else if status.contains("Not authenticated") {
        println!();
        println!("Run 'raworc auth' to authenticate with a server.");
    }

    Ok(())
}

async fn connect_to_server() -> Result<()> {
    print_banner();

    // Show authentication status below banner
    let status = get_auth_status().await?;
    println!("{status}");

    // Check if we can connect based on status
    if status.contains("Not authenticated") {
        println!();
        println!("Run 'raworc auth' to authenticate with a server.");
        return Ok(());
    }

    if status.contains("Not valid") {
        println!();
        println!("Run 'raworc auth' to re-authenticate.");
        return Ok(());
    }

    if status.contains("Not reachable") {
        println!();
        println!("Cannot connect to server. Please check the server status.");
        return Ok(());
    }

    // Get the server URL from auth config
    let server_url = match load_auth_config()? {
        Some(config) => config.server,
        None => {
            println!();
            println!("Run 'raworc auth' to authenticate with a server.");
            return Ok(());
        }
    };

    println!();

    // Start interactive loop with auto-completion
    let helper = RaworcHelper;
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));
    println!();

    while let Ok(line) = rl.readline("raworc> ") {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        rl.add_history_entry(line).ok();

        match line {
            "/quit" | "/q" | "q" | "quit" | "exit" => {
                break;
            }
            "/help" => {
                show_connect_help();
                println!();
            }
            "/status" => {
                let status = get_auth_status().await?;
                println!(" Authentication Status:");
                println!(" {status}");
                println!();
            }
            line if line.starts_with("/api ") => {
                let parts = &line[5..]; // Remove "/api "
                execute_api_request(&server_url, parts).await?;
                println!();
            }
            _ => {
                println!("Unknown command. Type /help for available commands.");
                println!();
            }
        }
    }

    Ok(())
}

fn show_connect_help() {
    println!(" Available Commands:");
    println!();
    println!("  /api <METHOD> <endpoint> [json]  - Execute REST API request");
    println!("  /api <endpoint>                  - Execute GET request (shorthand)");
    println!("  /status                          - Show server status");
    println!("  /help                            - Show this help");
    println!("  /quit, /q, q, quit, exit         - Exit interactive mode");
    println!();
    println!(" Examples:");
    println!("  /api version                     - GET /api/v0/version");
    println!("  /api service-accounts            - GET /api/v0/service-accounts");
    println!("  /api GET roles                   - GET /api/v0/roles");
    println!("  /api POST roles {{\"name\":\"test\",\"rules\":[]}}");
    println!("  /api DELETE roles/test-role");
    println!("  /api PUT service-accounts/admin {{\"description\":\"Updated\"}}");
}

async fn execute_api_request(server_url: &str, input: &str) -> Result<()> {
    // Check authentication using same logic
    let config = match load_auth_config()? {
        Some(config) => {
            if config.server != server_url {
                println!("✗ Not authenticated for this server. Use 'raworc auth' first.");
                return Ok(());
            }
            config
        }
        None => {
            println!("✗ Authentication required. Use 'raworc auth' first.");
            return Ok(());
        }
    };

    // Parse the command
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        println!("✗ No endpoint specified");
        return Ok(());
    }

    let (method, endpoint, data) = if parts[0].to_uppercase() == "GET"
        || parts[0].to_uppercase() == "POST"
        || parts[0].to_uppercase() == "PUT"
        || parts[0].to_uppercase() == "DELETE"
    {
        // Format: METHOD endpoint [data]
        if parts.len() < 2 {
            println!("✗ No endpoint specified after method");
            return Ok(());
        }
        let method = parts[0].to_uppercase();
        let endpoint = parts[1];
        let data = if parts.len() > 2 {
            // Join remaining parts as JSON data
            Some(parts[2..].join(" "))
        } else {
            None
        };
        (method, endpoint, data)
    } else {
        // Format: endpoint (assumes GET)
        ("GET".to_string(), parts[0], None)
    };

    let client = reqwest::Client::new();
    let url = format!("{server_url}/api/v0/{endpoint}");
    
    let mut request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        _ => {
            println!("✗ Unsupported HTTP method: {method}");
            return Ok(());
        }
    };

    request = request.header("Authorization", format!("Bearer {}", config.token));

    // Add JSON body if provided
    if let Some(json_data) = data {
        // Validate JSON
        match serde_json::from_str::<Value>(&json_data) {
            Ok(_) => {
                request = request
                    .header("Content-Type", "application/json")
                    .body(json_data);
            }
            Err(e) => {
                println!("✗ Invalid JSON data: {e}");
                return Ok(());
            }
        }
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            println!(" {method} {endpoint} → {status}");
            
            if let Ok(text) = response.text().await {
                if !text.is_empty() {
                    // Try to parse and pretty-print JSON
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            println!(" Response:");
                            let pretty = serde_json::to_string_pretty(&json)?;
                            for line in pretty.lines() {
                                println!("  {line}");
                            }
                        }
                        Err(_) => {
                            // Not JSON, print as-is
                            println!(" Response: {text}");
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to execute API request: {e}");
        }
    }
    Ok(())
}

fn print_banner() {
    println!();
    println!("╭──────────────────────────────────────────────────╮");
    println!("│ ❋ Welcome to Raworc!                             │");
    println!("│                                                  │");
    println!("│   Remote Agent Work Orchestration                │");
    println!("│                                                  │");
    println!("│   Type /help for commands, /quit or q to exit    │");
    println!("╰──────────────────────────────────────────────────╯");
    println!();
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
    println!("    help               Show this help message");
}
