use crate::models::todo::{CreateTodo, Todo};
use sqlx::PgPool;
use std::sync::Arc;
use uuid;

#[async_trait::async_trait]
pub trait TodoRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Todo>, anyhow::Error>;
    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Todo, anyhow::Error>;
    async fn update_with_id(&self, id: uuid::Uuid, todo: &Todo) -> Result<u64, anyhow::Error>;
    async fn insert_todo(&self, todo: &CreateTodo) -> Result<Todo, anyhow::Error>;
    async fn delete_from_id(&self, id: uuid::Uuid) -> Result<u64, anyhow::Error>;
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn TodoRepository>,
}

#[async_trait::async_trait]
impl TodoRepository for PgPool {
    async fn get_all(&self) -> Result<Vec<Todo>, anyhow::Error> {
        let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
            .fetch_all(self)
            .await?;
        Ok(todos)
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Todo, anyhow::Error> {
        let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE public_id = $1")
            .bind(id)
            .fetch_one(self)
            .await?;
        Ok(todo)
    }
    async fn update_with_id(&self, id: uuid::Uuid, todo: &Todo) -> Result<u64, anyhow::Error> {
        let result = sqlx::query("UPDATE todos SET text=$1, completed=$2 WHERE public_id=$3")
            .bind(&todo.text)
            .bind(todo.completed)
            .bind(id)
            .execute(self)
            .await?;
        Ok(result.rows_affected())
    }
    async fn insert_todo(&self, todo: &CreateTodo) -> Result<Todo, anyhow::Error> {
        let todo = sqlx::query_as::<_, Todo>(
            "INSERT INTO todos (text, completed) VALUES ($1, $2) RETURNING public_id, id, text, completed",
        )
        .bind(&todo.text)
        .bind(false) //todo.completed
        .fetch_one(self)
        .await?;
        Ok(todo)
    }
    async fn delete_from_id(&self, id: uuid::Uuid) -> Result<u64, anyhow::Error> {
        let result = sqlx::query("DELETE FROM todos WHERE public_id = $1")
            .bind(id)
            .execute(self)
            .await?;
        Ok(result.rows_affected())
    }
}
