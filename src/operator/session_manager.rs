use anyhow::Result;
use bollard::Docker;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::docker_manager::DockerManager;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct SessionTask {
    id: Uuid,
    task_type: String,
    session_id: Uuid,
    payload: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    error: Option<String>,
}

pub struct SessionManager {
    pool: Pool<Postgres>,
    docker_manager: DockerManager,
}

impl SessionManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let docker = Docker::connect_with_socket_defaults()?;
        let docker_manager = DockerManager::new(docker);

        Ok(Self {
            pool,
            docker_manager,
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Session Manager started, polling for tasks...");

        loop {
            match self.process_pending_tasks().await {
                Ok(processed) => {
                    if processed == 0 {
                        sleep(Duration::from_secs(2)).await;
                    }
                }
                Err(e) => {
                    error!("Error processing tasks: {}", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn process_pending_tasks(&self) -> Result<usize> {
        let tasks = self.fetch_pending_tasks().await?;
        let mut processed = 0;

        for task in tasks {
            match self.process_task(task).await {
                Ok(_) => processed += 1,
                Err(e) => error!("Failed to process task: {}", e),
            }
        }

        Ok(processed)
    }

    async fn fetch_pending_tasks(&self) -> Result<Vec<SessionTask>> {
        let tasks = sqlx::query_as::<_, SessionTask>(
            r#"
            UPDATE session_tasks
            SET status = 'processing',
                started_at = NOW(),
                updated_at = NOW()
            WHERE id IN (
                SELECT id
                FROM session_tasks
                WHERE status = 'pending'
                ORDER BY created_at
                LIMIT 5
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tasks)
    }

    async fn process_task(&self, task: SessionTask) -> Result<()> {
        info!("Processing task {} of type {}", task.id, task.task_type);

        let result = match task.task_type.as_str() {
            "create_session" => self.handle_create_session(task.clone()).await,
            "destroy_session" => self.handle_destroy_session(task.clone()).await,
            "execute_command" => self.handle_execute_command(task.clone()).await,
            _ => {
                warn!("Unknown task type: {}", task.task_type);
                Err(anyhow::anyhow!("Unknown task type"))
            }
        };

        match result {
            Ok(_) => {
                self.mark_task_completed(task.id).await?;
                info!("Task {} completed successfully", task.id);
            }
            Err(e) => {
                self.mark_task_failed(task.id, &e.to_string()).await?;
                error!("Task {} failed: {}", task.id, e);
            }
        }

        Ok(())
    }

    async fn handle_create_session(&self, task: SessionTask) -> Result<()> {
        let session_id = task.session_id;
        
        info!("Creating container for session {}", session_id);
        self.docker_manager.create_container(session_id).await?;
        
        sqlx::query(
            "UPDATE sessions SET state = 'READY', started_at = NOW(), last_activity_at = NOW() WHERE id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn handle_destroy_session(&self, task: SessionTask) -> Result<()> {
        let session_id = task.session_id;
        
        info!("Destroying container for session {}", session_id);
        self.docker_manager.destroy_container(session_id).await?;
        
        sqlx::query(
            "UPDATE sessions SET state = 'IDLE', terminated_at = NOW() WHERE id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn handle_execute_command(&self, task: SessionTask) -> Result<()> {
        let session_id = task.session_id;
        let command = task.payload["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing command in payload"))?;
        
        info!("Executing command in session {}: {}", session_id, command);
        let output = self.docker_manager.execute_command(session_id, command).await?;
        
        sqlx::query(
            r#"
            INSERT INTO command_results (id, session_id, command, output, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(session_id)
        .bind(command)
        .bind(output)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn mark_task_completed(&self, task_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE session_tasks
            SET status = 'completed',
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_task_failed(&self, task_id: Uuid, error: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE session_tasks
            SET status = 'failed',
                error = $2,
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(task_id)
        .bind(error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}