use crate::models::todo::Todo;
use crate::routes::health::health_check;
use crate::routes::todos::{todos_create, todos_delete, todos_index, todos_update};

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    http::StatusCode,
    routing::{get, patch},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

const REQUEST_TIMEOUT_SECONDS: u64 = 10;

pub fn app() -> Router {
    let db = Db::default();
    Router::new()
        .route("/health", get(health_check))
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/{id}", patch(todos_update).delete(todos_delete))
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
        .with_state(db)
}

type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;
