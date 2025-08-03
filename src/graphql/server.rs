use anyhow::Result;
use async_graphql::{EmptySubscription, Schema};
use axum::{extract::Extension, routing::post, Router};
use std::fs;
use std::process;
use tracing::{error, info, warn};

use crate::database::{initialize_app_state, seed_rbac_system};
use crate::graphql::{graphql_handler, Mutation, Query};

pub async fn run_graphql_server() -> Result<()> {
    // Write PID file for process management
    let pid = process::id();
    let pid_file = "/tmp/raworc.pid";

    if let Err(e) = fs::write(pid_file, pid.to_string()) {
        warn!("Could not write PID file: {}", e);
    }

    // Set up cleanup on exit
    let pid_file_cleanup = pid_file.to_string();
    ctrlc::set_handler(move || {
        info!("Shutting down Raworc server...");
        let _ = fs::remove_file(&pid_file_cleanup);
        std::process::exit(0);
    })?;

    // Log startup banner
    info!(
        r#"
 ____                                
|  _ \ __ ___      _____  _ __ ___ 
| |_) / _` \ \ /\ / / _ \| '__/ __|
|  _ < (_| |\ V  V / (_) | | | (__ 
|_| \_\__,_| \_/\_/ \___/|_|  \___|
                                  
Starting Raworc GraphQL service...
PID: {}
"#,
        pid
    );

    // Initialize database connection and app state
    info!("Connecting to PostgreSQL database...");
    
    // Get environment variables or use defaults
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-key".to_string());
    
    let app_state = match initialize_app_state(&database_url, jwt_secret).await {
        Ok(state) => {
            info!("Connected to database successfully!");
            state
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            error!("Please ensure PostgreSQL is running and DATABASE_URL is set correctly");
            error!("Example: DATABASE_URL=postgresql://user:password@host:port/database");
            return Err(anyhow::anyhow!(
                "Database not available. Please check your configuration."
            ));
        }
    };

    // Seed RBAC system if service_accounts table is empty
    if let Err(e) = seed_rbac_system(&app_state).await {
        error!("Failed to seed RBAC system: {}", e);
    }

    // Build GraphQL schema with security configurations
    info!("Building secure GraphQL schema...");
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(app_state.clone())
        .limit_depth(10) // Depth limiting
        .limit_complexity(1000) // Query complexity limiting
        .finish();

    // Build HTTP router
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .layer(Extension(schema))
        .layer(Extension(app_state));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await?;

    info!("Server started successfully!");
    info!("GraphQL Endpoint: http://localhost:9000/graphql");
    info!("Ready to accept requests...");

    let result = axum::serve(listener, app).await;

    // Clean up PID file on exit
    let _ = fs::remove_file(pid_file);

    result?;
    Ok(())
}
