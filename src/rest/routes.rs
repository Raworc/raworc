use axum::{
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::models::AppState;
use crate::rest::{auth, handlers, middleware::auth_middleware, logging_middleware::request_logging_middleware, openapi::ApiDoc};

pub fn create_router(state: Arc<AppState>) -> Router {
    // Public routes
    let public_routes = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/auth/login", post(auth::login))
        .route("/auth/external-login", post(auth::external_login));
    
    // Protected routes
    let protected_routes = Router::new()
        .route("/auth/me", get(auth::me))
        // Service account endpoints
        .route("/service-accounts", get(handlers::service_accounts::list_service_accounts))
        .route("/service-accounts", post(handlers::service_accounts::create_service_account))
        .route("/service-accounts/{id}", get(handlers::service_accounts::get_service_account))
        .route("/service-accounts/{id}", delete(handlers::service_accounts::delete_service_account))
        // Role endpoints
        .route("/roles", get(handlers::roles::list_roles))
        .route("/roles", post(handlers::roles::create_role))
        .route("/roles/{id}", get(handlers::roles::get_role))
        .route("/roles/{id}", delete(handlers::roles::delete_role))
        // Role binding endpoints
        .route("/role-bindings", get(handlers::role_bindings::list_role_bindings))
        .route("/role-bindings", post(handlers::role_bindings::create_role_binding))
        .route("/role-bindings/{id}", get(handlers::role_bindings::get_role_binding))
        .route("/role-bindings/{id}", delete(handlers::role_bindings::delete_role_binding))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let api_routes = public_routes.merge(protected_routes).with_state(state.clone());

    Router::new()
        .nest("/api/v1", api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn version() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "version": "0.1.0",
        "api": "v1"
    }))
}