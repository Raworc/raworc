use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub server: String,
    pub token: String,
}

// Directory and config management
fn get_raworc_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".raworc"))
}

fn get_config_file() -> Result<PathBuf> {
    let raworc_dir = get_raworc_dir()?;
    Ok(raworc_dir.join("auth.yaml"))
}

pub async fn store_auth_config(server_url: &str, token: &str) -> Result<()> {
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

pub fn load_auth_config() -> Result<Option<AuthConfig>> {
    let config_file = get_config_file()?;
    match fs::read_to_string(config_file) {
        Ok(content) => match serde_yaml::from_str::<AuthConfig>(&content) {
            Ok(config) => Ok(Some(config)),
            Err(_) => Ok(None),
        },
        Err(_) => Ok(None),
    }
}

pub async fn validate_token(server_url: &str, token: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();

    match client
        .get(format!("{server_url}/api/v0/auth/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let result: serde_json::Value = response.json().await?;
            Ok(result
                .get("user")
                .or_else(|| result.get("name"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string()))
        }
        _ => Ok(None),
    }
}

pub async fn auth_interactive() -> Result<()> {
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

pub async fn auth_login() -> Result<()> {
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
    let login_request = serde_json::json!({
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
        let result: serde_json::Value = response.json().await?;
        if let Some(token) = result.get("token").and_then(|t| t.as_str()) {
            store_auth_config(server_url, token).await?;
            if let Some(user) = validate_token(server_url, token).await? {
                println!();
                println!("✓ Authentication successful!");
                println!("   User: {user}");
                println!("   Server: {server_url}");
                println!();
                println!("You can now use 'raworc' or 'raworc connect' to connect to this server.");
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

pub async fn auth_token_interactive() -> Result<()> {
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
        println!("You can now use 'raworc' or 'raworc connect' to connect to this server.");
    } else {
        println!("✗ Invalid token or server unreachable");
    }
    Ok(())
}

pub async fn get_auth_status() -> Result<String> {
    // Check if auth config exists
    match load_auth_config()? {
        Some(config) => {
            // Check server reachability using REST endpoint
            let client = reqwest::Client::new();
            let server_reachable = match client
                .get(format!("{}/api/v0/health", config.server))
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
                        "✓ Authenticated as: {user}\n   Server: {}",
                        config.server
                    ))
                } else {
                    Ok(format!(
                        "✗ Token is not valid\n   Server: {}",
                        config.server
                    ))
                }
            } else {
                Ok(format!(
                    "✗ Server is not reachable\n   Server: {}",
                    config.server
                ))
            }
        }
        None => Ok("✗ Not authenticated. Run 'raworc auth' to authenticate.".to_string()),
    }
}

pub async fn show_auth_status() -> Result<()> {
    let status = get_auth_status().await?;
    println!();
    println!("Authentication Status:");
    println!("{status}");
    println!();
    Ok(())
}