use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::info;

pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();
    
    // Get user info if available (for authenticated endpoints)
    let user = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|_| "authenticated")
        .unwrap_or("anonymous");
    
    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();
    
    // Log the request with response status and duration
    info!(
        method = %method,
        path = %uri.path(),
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        user = %user,
        "HTTP request"
    );
    
    Ok(response)
}