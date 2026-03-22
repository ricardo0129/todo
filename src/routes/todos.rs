use crate::models::todo::{CreateTodo, Todo, UpdateTodo};
use crate::routes::appstate::AppState;
use axum::extract::{Path, Query};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

pub async fn todos_create(
    State(appstate): State<AppState>,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = Todo::create_todo(input.text, false);
    let result = appstate.db.insert_todo(&todo).await;

    match result {
        Ok(todo) => (StatusCode::CREATED, Json(todo)),
        Err(e) => {
            println!("error inserting {e}");
            (StatusCode::FORBIDDEN, Json(todo))
        }
    }
}

// The query parameters for todos index
#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

pub async fn todos_index(
    pagination: Query<Pagination>,
    State(appstate): State<AppState>,
) -> impl IntoResponse {
    let todos = appstate.db.get_all().await.expect("couldn't read");

    let todos = todos
        .iter()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();

    Json(todos)
}
pub async fn todos_update(
    Path(id): Path<uuid::Uuid>,
    State(appstate): State<AppState>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo: Todo = appstate.db.get_by_id(id).await.expect("failed to find ");
    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }
    let result = appstate.db.update_with_id(id, &todo).await;
    match result {
        Ok(_) => Ok(Json(todo)),
        Err(_) => {
            println!("Unable to update");
            Ok(Json(todo))
        }
    }
}

pub async fn todos_delete(
    Path(id): Path<uuid::Uuid>,
    State(appstate): State<AppState>,
) -> impl IntoResponse {
    let result = appstate.db.delete_from_id(id).await;
    match result {
        Ok(0) => StatusCode::NOT_FOUND,
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::FORBIDDEN,
    }
}

#[cfg(test)]
mod tests {
    use crate::app::build_app;
    use crate::models::todo::Todo;
    use crate::routes::appstate::{AppState, TodoRepository};
    use axum::Router;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`
    #[derive(Clone)]
    struct MockRepository;

    #[async_trait::async_trait]
    impl TodoRepository for MockRepository {
        async fn get_all(&self) -> Result<Vec<Todo>, anyhow::Error> {
            Ok(vec![Todo {
                id: 1,
                public_id: uuid::Uuid::from_u64_pair(128, 4),
                text: "Mock".into(),
                completed: false,
            }])
        }

        async fn get_by_id(&self, id: uuid::Uuid) -> Result<Todo, anyhow::Error> {
            Ok(Todo {
                id: 1,
                public_id: id,
                text: "Mock".into(),
                completed: false,
            })
        }
        async fn update_with_id(
            &self,
            _id: uuid::Uuid,
            _todo: &Todo,
        ) -> Result<u64, anyhow::Error> {
            Ok(1)
        }
        async fn insert_todo(&self, todo: &Todo) -> Result<Todo, anyhow::Error> {
            Ok(Todo {
                id: 1,
                public_id: uuid::Uuid::from_u64_pair(128, 4),
                text: todo.text.clone(),
                completed: todo.completed,
            })
        }
        async fn delete_from_id(&self, _id: uuid::Uuid) -> Result<u64, anyhow::Error> {
            Ok(3)
        }
    }

    async fn build_test_router() -> Router {
        let state = AppState {
            db: Arc::new(MockRepository),
        };
        build_app(state).await
    }

    #[tokio::test]
    async fn get_todos() {
        let app = build_test_router().await;
        let response = app
            .oneshot(Request::get("/todos").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
