use anyhow::{Result, Context};
use bollard::{Docker, API_DEFAULT_VERSION};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, 
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    LogsOptions, StatsOptions, InspectContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::models::{ContainerInspectResponse, ContainerStateStatusEnum};
use bollard::container::Stats;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::default::Default;
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub socket_path: Option<String>,
    pub version: String,
}

impl Default for DockerConfig {
    fn default() -> Self {
        Self {
            socket_path: None,  // Will use default socket
            version: API_DEFAULT_VERSION.to_string(),
        }
    }
}

pub struct DockerClient {
    pub(super) docker: Docker,
}

impl DockerClient {
    pub async fn new(config: DockerConfig) -> Result<Self> {
        let docker = if let Some(socket) = config.socket_path {
            Docker::connect_with_socket(&socket, 120, &API_DEFAULT_VERSION)?
        } else {
            Docker::connect_with_socket_defaults()?
        };
        
        // Test connection
        let version = docker.version().await
            .context("Failed to connect to Docker daemon")?;
        
        info!("Connected to Docker daemon version: {}", version.version.unwrap_or_default());
        
        Ok(Self { docker })
    }
    
    pub async fn pull_image(&self, image: &str) -> Result<()> {
        info!("Pulling Docker image: {}", image);
        
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };
        
        let mut stream = self.docker.create_image(Some(options), None, None);
        
        while let Some(info) = stream.next().await {
            match info {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Pull status: {}", status);
                    }
                }
                Err(e) => {
                    error!("Error pulling image: {}", e);
                    return Err(anyhow::anyhow!("Failed to pull image: {}", e));
                }
            }
        }
        
        info!("Successfully pulled image: {}", image);
        Ok(())
    }
    
    pub async fn create_container(
        &self,
        name: &str,
        config: Config<String>,
    ) -> Result<String> {
        let options = CreateContainerOptions {
            name,
            platform: None,
        };
        
        let response = self.docker
            .create_container(Some(options), config)
            .await
            .context("Failed to create container")?;
        
        info!("Created container {} with ID: {}", name, response.id);
        Ok(response.id)
    }
    
    pub async fn start_container(&self, id: &str) -> Result<()> {
        self.docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await
            .context("Failed to start container")?;
        
        info!("Started container: {}", id);
        Ok(())
    }
    
    pub async fn stop_container(&self, id: &str, timeout: Option<i64>) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10),
        };
        
        self.docker
            .stop_container(id, Some(options))
            .await
            .context("Failed to stop container")?;
        
        info!("Stopped container: {}", id);
        Ok(())
    }
    
    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        
        self.docker
            .remove_container(id, Some(options))
            .await
            .context("Failed to remove container")?;
        
        info!("Removed container: {}", id);
        Ok(())
    }
    
    pub async fn inspect_container(&self, id: &str) -> Result<ContainerInspectResponse> {
        self.docker
            .inspect_container(id, None::<InspectContainerOptions>)
            .await
            .context("Failed to inspect container")
    }
    
    pub async fn container_exists(&self, id: &str) -> bool {
        self.inspect_container(id).await.is_ok()
    }
    
    pub async fn is_container_running(&self, id: &str) -> Result<bool> {
        let info = self.inspect_container(id).await?;
        
        Ok(info.state
            .and_then(|s| s.status)
            .map(|s| s == ContainerStateStatusEnum::RUNNING)
            .unwrap_or(false))
    }
    
    pub async fn list_containers(&self, all: bool) -> Result<Vec<bollard::models::ContainerSummary>> {
        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };
        
        self.docker
            .list_containers(Some(options))
            .await
            .context("Failed to list containers")
    }
    
    pub async fn exec_command(
        &self,
        container_id: &str,
        cmd: Vec<String>,
    ) -> Result<String> {
        let exec_config = CreateExecOptions {
            cmd: Some(cmd),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };
        
        let exec = self.docker
            .create_exec(container_id, exec_config)
            .await
            .context("Failed to create exec")?;
        
        let start_exec = self.docker
            .start_exec(&exec.id, None)
            .await
            .context("Failed to start exec")?;
        
        let mut result = String::new();
        
        if let StartExecResults::Attached { mut output, .. } = start_exec {
            while let Some(msg) = output.next().await {
                match msg {
                    Ok(msg) => result.push_str(&msg.to_string()),
                    Err(e) => warn!("Error reading exec output: {}", e),
                }
            }
        }
        
        Ok(result)
    }
    
    pub async fn get_container_logs(
        &self,
        container_id: &str,
        tail: Option<usize>,
    ) -> Result<String> {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            tail: tail.map(|t| t.to_string()).unwrap_or_else(|| "all".to_string()),
            ..Default::default()
        };
        
        let mut stream = self.docker.logs(container_id, Some(options));
        let mut logs = String::new();
        
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => logs.push_str(&msg.to_string()),
                Err(e) => warn!("Error reading logs: {}", e),
            }
        }
        
        Ok(logs)
    }
    
    pub async fn get_container_stats(&self, container_id: &str) -> Result<Stats> {
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        
        let mut stream = self.docker.stats(container_id, Some(options));
        
        if let Some(stats) = stream.next().await {
            stats.context("Failed to get container stats")
        } else {
            Err(anyhow::anyhow!("No stats available"))
        }
    }
    
    pub async fn wait_for_container(&self, container_id: &str) -> Result<i64> {
        let mut stream = self.docker.wait_container(container_id, None::<bollard::container::WaitContainerOptions<String>>);
        
        if let Some(result) = stream.next().await {
            match result {
                Ok(wait_response) => Ok(wait_response.status_code),
                Err(e) => Err(anyhow::anyhow!("Failed to wait for container: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("Container wait stream ended unexpectedly"))
        }
    }
}