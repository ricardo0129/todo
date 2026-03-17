use crate::models::todo::Todo;
use crate::routes::health::health_check;
use crate::routes::todos::{todos_create, todos_delete, todos_index};

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{delete, get, patch},
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

const REQUEST_TIMEOUT_SECONDS: u64 = 10;

pub async fn app() -> Router {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:mysecretpassword@localhost".to_string());

    let pool: PgPool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to db");

    Router::new()
        .route("/health", get(health_check))
        .route("/todos", get(todos_index).post(todos_create))
        //.route("/todos/{id}", patch(todos_update).delete(todos_delete))
        .route("/todos/{id}", delete(todos_delete))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(pool)
}
