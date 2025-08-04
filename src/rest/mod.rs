pub mod auth;
pub mod error;
pub mod handlers;
pub mod logging_middleware;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod server;

pub use error::{ApiError, ApiResult};
pub use routes::create_router;
pub use server::run_rest_server;