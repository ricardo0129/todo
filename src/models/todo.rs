use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Debug, Serialize, Clone)]
pub struct Todo {
    pub id: i64,
    pub text: String,
    pub completed: bool,
}

#[derive(FromRow, Debug, Deserialize)]
pub struct CreateTodo {
    pub text: String,
}

#[derive(FromRow, Debug, Deserialize)]
pub struct UpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
}
