use anyhow::Result;
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions},
    exec::{CreateExecOptions, StartExecResults},
    Docker,
};
use futures::StreamExt;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;

pub struct DockerManager {
    docker: Docker,
    host_image: String,
    cpu_limit: f64,
    memory_limit: i64,
}

impl DockerManager {
    pub fn new(docker: Docker) -> Self {
        Self {
            docker,
            host_image: std::env::var("HOST_AGENT_IMAGE")
                .unwrap_or_else(|_| "raworc-host:latest".to_string()),
            cpu_limit: std::env::var("HOST_AGENT_CPU_LIMIT")
                .unwrap_or_else(|_| "0.5".to_string())
                .parse()
                .unwrap_or(0.5),
            memory_limit: std::env::var("HOST_AGENT_MEMORY_LIMIT")
                .unwrap_or_else(|_| "536870912".to_string())
                .parse()
                .unwrap_or(536870912),
        }
    }

    pub async fn create_container(&self, session_id: Uuid) -> Result<String> {
        let container_name = format!("raworc-session-{}", session_id);
        
        info!("Creating container {} with image {}", container_name, self.host_image);

        let mut labels = HashMap::new();
        labels.insert("raworc.session".to_string(), session_id.to_string());
        labels.insert("raworc.managed".to_string(), "true".to_string());

        // Set environment variables for the host agent
        let env = vec![
            format!("RAWORC_API_URL=http://raworc-server:9000"),
            format!("RAWORC_SESSION_ID={}", session_id),
            format!("RAWORC_API_KEY=session-{}", session_id),  // TODO: Generate proper API key
        ];

        let config = Config {
            image: Some(self.host_image.clone()),
            hostname: Some(format!("session-{}", &session_id.to_string()[..8])),
            labels: Some(labels),
            env: Some(env),
            host_config: Some(bollard::models::HostConfig {
                cpu_quota: Some((self.cpu_limit * 100000.0) as i64),
                cpu_period: Some(100000),
                memory: Some(self.memory_limit),
                memory_swap: Some(self.memory_limit),
                network_mode: Some("raworc-network".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let container = self.docker.create_container(Some(options), config).await?;
        
        self.docker
            .start_container::<String>(&container.id, None)
            .await?;

        info!("Container {} created and started", container_name);
        Ok(container.id)
    }

    pub async fn destroy_container(&self, session_id: Uuid) -> Result<()> {
        let container_name = format!("raworc-session-{}", session_id);
        
        info!("Destroying container {}", container_name);

        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };

        match self.docker.remove_container(&container_name, Some(options)).await {
            Ok(_) => {
                info!("Container {} destroyed", container_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to destroy container {}: {}", container_name, e);
                Err(anyhow::anyhow!("Failed to destroy container: {}", e))
            }
        }
    }

    pub async fn execute_command(&self, session_id: Uuid, command: &str) -> Result<String> {
        let container_name = format!("raworc-session-{}", session_id);
        
        info!("Executing command in container {}: {}", container_name, command);

        let exec_config = CreateExecOptions {
            cmd: Some(vec!["/bin/bash", "-c", command]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self.docker
            .create_exec(&container_name, exec_config)
            .await?;

        let mut output_str = String::new();
        
        if let StartExecResults::Attached { mut output, .. } = 
            self.docker.start_exec(&exec.id, None).await? 
        {
            while let Some(Ok(msg)) = output.next().await {
                output_str.push_str(&msg.to_string());
            }
        }

        Ok(output_str)
    }
}