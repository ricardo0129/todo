use crate::models::todo::{CreateTodo, Todo, UpdateTodo};
use axum::extract::{Path, Query};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::postgres::PgPool;

pub async fn todos_create(
    State(db): State<PgPool>,
    Json(input): Json<CreateTodo>,
) -> impl IntoResponse {
    let todo = Todo {
        id: -1,
        text: input.text,
        completed: false,
    };
    let result = sqlx::query("INSERT INTO todos (text, completed) VALUES ($1, $2)")
        .bind(&todo.text)
        .bind(todo.completed)
        .execute(&db)
        .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(todo)),
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
    State(db): State<PgPool>,
) -> impl IntoResponse {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&db)
        .await
        .expect("couldn't read");

    let todos = todos
        .iter()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();

    Json(todos)
}
pub async fn todos_update(
    Path(id): Path<i64>,
    State(db): State<PgPool>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo: Todo = sqlx::query_as::<_, Todo>("SELECT * from todos WHERE id = $1")
        .bind(id)
        .fetch_one(&db)
        .await
        .expect("failed to find ");
    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }
    let result = sqlx::query("UPDATE todos SET text=$1, completed=$2 WHERE id=$3")
        .bind(&todo.text)
        .bind(todo.completed)
        .bind(id)
        .execute(&db)
        .await;
    match result {
        Ok(_) => Ok(Json(todo)),
        Err(_) => {
            println!("Unable to update");
            Ok(Json(todo))
        }
    }
}

pub async fn todos_delete(Path(id): Path<i64>, State(db): State<PgPool>) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&db)
        .await;
    match result {
        Ok(rows) if rows.rows_affected() == 0 => StatusCode::NOT_FOUND,
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::FORBIDDEN,
    }
}

#[cfg(test)]
mod tests {
    use crate::app::app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`
    #[tokio::test]
    async fn get_todos() {
        let app = app().await;
        let response = app
            .oneshot(Request::get("/todos").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
