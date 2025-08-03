use thiserror::Error;
use sqlx::{Pool, Postgres};

// Database errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("General error: {0}")]
    General(#[from] anyhow::Error),
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<Pool<Postgres>>,
    pub jwt_secret: String,
}
