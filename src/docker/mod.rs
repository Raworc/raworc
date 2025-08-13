mod client;
mod container;
mod volume;
mod lifecycle;

pub use client::DockerClient;
pub use container::ContainerStatus;
pub use lifecycle::ContainerLifecycleManager;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerSessionConfig {
    pub image: String,
    pub cpu_limit: f64,      // Number of CPUs (e.g., 0.5 for half a CPU)
    pub memory_limit: i64,    // Memory in bytes
    pub disk_limit: i64,      // Disk in bytes
    pub network: Option<String>,
    pub volumes_path: String,
}

impl Default for DockerSessionConfig {
    fn default() -> Self {
        Self {
            image: "python:3.11-slim".to_string(),
            cpu_limit: 0.5,
            memory_limit: 512 * 1024 * 1024,  // 512MB
            disk_limit: 1024 * 1024 * 1024,    // 1GB
            network: None,
            volumes_path: "/var/lib/raworc/volumes".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContainerEvent {
    Created { session_id: Uuid, container_id: String },
    Started { session_id: Uuid, container_id: String },
    Stopped { session_id: Uuid, container_id: String },
    Removed { session_id: Uuid, container_id: String },
    Failed { session_id: Uuid, container_id: String, reason: String },
}