use anyhow::Result;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::cli_auth::{get_auth_status, load_auth_config};

pub async fn connect_to_server() -> Result<()> {
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

    if status.contains("not reachable") {
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

    // Start interactive loop
    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    
    println!("Type /help for available commands");
    println!();

    loop {
        match rl.readline("raworc> ") {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line)?;

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
            Err(ReadlineError::Interrupted) => {
                println!("Use /quit to exit");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
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

fn show_connect_help() {
    println!(" Available Commands:");
    println!();
    println!("  /api <METHOD> <endpoint> [json]  - Execute REST API request");
    println!("  /api <endpoint>                  - Execute GET request (shorthand)");
    println!("  /status                          - Show authentication status");
    println!("  /help                            - Show this help");
    println!("  /quit, /q, q, quit, exit         - Exit interactive mode");
    println!();
    println!(" Examples:");
    println!("  /api health                      - GET /api/v0/health");
    println!("  /api agents                      - GET /api/v0/agents");
    println!("  /api sessions                    - GET /api/v0/sessions");
    println!("  /api GET sessions                - GET /api/v0/sessions");
    println!("  /api POST agents {{\"name\":\"test\",\"model\":\"claude-3-haiku\"}}");
    println!("  /api DELETE sessions/uuid");
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
            println!("✗ Not authenticated. Use 'raworc auth' first.");
            return Ok(());
        }
    };

    let client = reqwest::Client::new();
    let parts: Vec<&str> = input.split_whitespace().collect();
    
    if parts.is_empty() {
        println!("Usage: /api <METHOD> <endpoint> [json]");
        return Ok(());
    }

    let (method, endpoint, body) = if parts[0].to_uppercase() == "GET"
        || parts[0].to_uppercase() == "POST"
        || parts[0].to_uppercase() == "PUT"
        || parts[0].to_uppercase() == "DELETE"
        || parts[0].to_uppercase() == "PATCH"
    {
        // Format: /api METHOD endpoint [body]
        let method = parts[0].to_uppercase();
        let endpoint = if parts.len() > 1 { parts[1] } else { "" };
        let body = if parts.len() > 2 {
            Some(parts[2..].join(" "))
        } else {
            None
        };
        (method, endpoint, body)
    } else {
        // Format: /api endpoint (defaults to GET)
        ("GET".to_string(), parts[0], None)
    };

    let url = if endpoint.starts_with("http") {
        endpoint.to_string()
    } else if endpoint.starts_with("/") {
        format!("{}{}", server_url, endpoint)
    } else {
        format!("{}/api/v0/{}", server_url, endpoint)
    };

    println!(" → {method} {url}");
    
    let mut request = client.request(
        method.parse::<reqwest::Method>()?,
        &url,
    )
    .header("Authorization", format!("Bearer {}", config.token));

    if let Some(body_str) = body {
        request = request
            .header("Content-Type", "application/json")
            .body(body_str);
    }

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            println!(" ← {status}");
            
            if let Ok(text) = response.text().await {
                // Try to pretty-print JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("{}", serde_json::to_string_pretty(&json)?);
                } else {
                    println!("{text}");
                }
            }
        }
        Err(e) => {
            println!(" ✗ Request failed: {e}");
        }
    }

    Ok(())
}