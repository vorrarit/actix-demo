use std::collections::HashMap;

use actix_web::{post, web, HttpResponse};
use sqlx::SqlitePool;
use crate::{todo::model::TodoItem, utils};
use anyhow::Context;
use serde::Serialize;

#[tracing::instrument]
#[post("/todo")]
pub async fn todo_add(pool: web::Data<SqlitePool>, todo_item: web::Json<TodoItem>) -> Result<HttpResponse, utils::error::ActixDemoError> {
    let mut conn = pool.acquire()
        .await
        .context("Error getting connection from pool")?;
    
    let result = sqlx::query("insert into todos (description) values (?1)")
        .bind(todo_item.description.clone())
        .execute(&mut *conn)
        .await
        .context("Error inserting todo")?;

    let mut res: ActixDemoResponse<HashMap<String, i64>> = ActixDemoResponse {
        data: HashMap::new()
    };
    res.data.insert("row_id".to_string(), result.last_insert_rowid());
    Ok(HttpResponse::Ok().json(res))
}

#[derive(Serialize, Debug)]
struct ActixDemoResponse<T> {
    data: T
}