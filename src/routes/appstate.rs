use crate::models::todo::Todo;
use sqlx::{PgPool, postgres::PgQueryResult};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait TodoRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Todo>, anyhow::Error>;
    async fn get_by_id(&self, id: i64) -> Result<Todo, anyhow::Error>;
    async fn update_with_id(&self, id: i64, todo: &Todo) -> Result<PgQueryResult, sqlx::Error>;
    async fn insert_todo(&self, todo: &Todo) -> Result<PgQueryResult, sqlx::Error>;
    async fn delete_from_id(&self, id: i64) -> Result<PgQueryResult, sqlx::Error>;
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

    async fn get_by_id(&self, id: i64) -> Result<Todo, anyhow::Error> {
        let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
            .bind(id)
            .fetch_one(self)
            .await?;
        Ok(todo)
    }
    async fn update_with_id(&self, id: i64, todo: &Todo) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query("UPDATE todos SET text=$1, completed=$2 WHERE id=$3")
            .bind(&todo.text)
            .bind(todo.completed)
            .bind(id)
            .execute(self)
            .await
    }
    async fn insert_todo(&self, todo: &Todo) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query("INSERT INTO todos (text, completed) VALUES ($1, $2)")
            .bind(&todo.text)
            .bind(todo.completed)
            .execute(self)
            .await
    }
    async fn delete_from_id(&self, id: i64) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query("DELETE FROM todos WHERE id = $1")
            .bind(id)
            .execute(self)
            .await
    }
}
