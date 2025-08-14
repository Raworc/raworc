mod shared;
mod server;
mod operator;
mod host;
mod builder;
mod cli_auth;
mod cli_connect;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "raworc")]
#[command(about = "Raworc - Container orchestration system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start services (both server and operator by default)
    Start {
        /// Components to start (server, operator, or both)
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,
        
        /// Host for API server
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,
        
        /// Port for API server
        #[arg(short, long, default_value = "9000")]
        port: u16,
    },
    
    /// Stop services
    Stop {
        /// Components to stop (server, operator, or both)
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,
    },
    
    /// Connect to server interactively (default command)
    Connect,
    
    /// Authenticate with the API server
    Auth,
    
    /// Show authentication status
    Status,
    
    /// Start the host agent (runs inside containers)
    Host {
        /// API server URL
        #[arg(long, env = "RAWORC_API_URL")]
        api_url: String,
        
        /// Session ID
        #[arg(long, env = "RAWORC_SESSION_ID")]
        session_id: String,
        
        /// API Key for authentication
        #[arg(long, env = "RAWORC_API_KEY")]
        api_key: String,
    },
    
    /// Run the API server (internal use)
    Server,
    
    /// Run the operator (internal use)
    Operator,
    
    /// Build Docker images for Raworc components
    Build {
        /// Components to build (server, operator, host, all)
        /// Can specify multiple: raworc build server operator
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,
        
        /// Docker image tag
        #[arg(short, long, default_value = "latest")]
        tag: String,
        
        /// Build without cache
        #[arg(long)]
        no_cache: bool,
        
        /// Push images to registry after building
        #[arg(short, long)]
        push: bool,
        
        /// Registry to push to (e.g., docker.io/myorg)
        #[arg(short, long)]
        registry: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let _ = shared::logging::init_logging("./logs", "raworc");
    
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Default to connect if no command provided
    let command = cli.command.unwrap_or(Commands::Connect);
    
    match command {
        Commands::Start { components, host: _, port: _ } => {
            use std::process::Command;
            
            let components = if components.is_empty() {
                vec![]  // Empty means all services
            } else {
                components
            };
            
            // Build docker-compose command
            let mut cmd = Command::new("docker");
            cmd.arg("compose").arg("up").arg("-d");
            
            // Add specific services if requested
            for component in &components {
                match component.as_str() {
                    "server" => cmd.arg("raworc-server"),
                    "operator" => cmd.arg("raworc-operator"),
                    "postgres" => cmd.arg("raworc-postgres"),
                    _ => {
                        tracing::warn!("Unknown component: {}. Valid options: server, operator, postgres", component);
                        continue;
                    }
                };
            }
            
            tracing::info!("Starting services with Docker Compose...");
            
            // Execute docker-compose
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("{}", stdout);
                        tracing::info!("Services started successfully");
                        
                        // Show running containers
                        let ps_cmd = Command::new("docker")
                            .args(&["compose", "ps"])
                            .output();
                        
                        if let Ok(ps_output) = ps_cmd {
                            println!("\nRunning services:");
                            println!("{}", String::from_utf8_lossy(&ps_output.stdout));
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        tracing::error!("Failed to start services: {}", stderr);
                        eprintln!("Error: {}", stderr);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute docker-compose: {}", e);
                    eprintln!("Error: Failed to execute docker-compose: {}", e);
                    eprintln!("Make sure Docker and Docker Compose are installed");
                }
            }
        }
        Commands::Stop { components } => {
            use std::process::Command;
            
            let components = if components.is_empty() {
                vec![]  // Empty means all services
            } else {
                components
            };
            
            if components.is_empty() {
                // Stop all services
                tracing::info!("Stopping all services with Docker Compose...");
                
                let mut cmd = Command::new("docker");
                cmd.args(&["compose", "down"]);
                
                match cmd.output() {
                    Ok(output) => {
                        if output.status.success() {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            println!("{}", stdout);
                            tracing::info!("All services stopped");
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            tracing::error!("Failed to stop services: {}", stderr);
                            eprintln!("Error: {}", stderr);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to execute docker-compose: {}", e);
                        eprintln!("Error: Failed to execute docker-compose: {}", e);
                    }
                }
            } else {
                // Stop specific services
                for component in components {
                    let service_name = match component.as_str() {
                        "server" => "raworc-server",
                        "operator" => "raworc-operator",
                        "postgres" => "raworc-postgres",
                        _ => {
                            tracing::warn!("Unknown component: {}. Valid options: server, operator, postgres", component);
                            continue;
                        }
                    };
                    
                    tracing::info!("Stopping {}...", service_name);
                    
                    let mut cmd = Command::new("docker");
                    cmd.args(&["compose", "stop", service_name]);
                    
                    match cmd.output() {
                        Ok(output) => {
                            if output.status.success() {
                                println!("Stopped {}", service_name);
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                eprintln!("Failed to stop {}: {}", service_name, stderr);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error stopping {}: {}", service_name, e);
                        }
                    }
                }
            }
        }
        Commands::Connect => {
            cli_connect::connect_to_server().await?;
        }
        Commands::Auth => {
            cli_auth::auth_interactive().await?;
        }
        Commands::Status => {
            cli_auth::show_auth_status().await?;
        }
        Commands::Host { api_url, session_id, api_key } => {
            host::run(&api_url, &session_id, &api_key).await?;
        }
        Commands::Server => {
            server::rest::server::run_rest_server().await?;
        }
        Commands::Operator => {
            operator::run().await?;
        }
        Commands::Build { 
            components, 
            tag, 
            no_cache, 
            push, 
            registry 
        } => {
            builder::run(components, tag, no_cache, push, registry).await?;
        }
    }
    
    Ok(())
}