use thiserror::Error;

// Database errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] surrealdb::Error),
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<surrealdb::Surreal<surrealdb::engine::remote::ws::Client>>,
    pub jwt_secret: String,
}
