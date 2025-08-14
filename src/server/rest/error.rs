use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    #[allow(dead_code)]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    
    #[error("Database error")]
    Database(#[from] crate::shared::models::DatabaseError),
    
    #[error("JWT error")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Bcrypt error")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "Authentication required".to_string()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.to_string()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.to_string()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "An internal error occurred".to_string()),
            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database operation failed".to_string()),
            ApiError::Jwt(_) => (StatusCode::UNAUTHORIZED, "JWT_ERROR", "Invalid or expired token".to_string()),
            ApiError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CRYPTO_ERROR", "Cryptographic operation failed".to_string()),
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message,
                details: None,
            },
        };

        (status, Json(error_response)).into_response()
    }
}