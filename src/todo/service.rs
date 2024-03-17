use actix_web::{post, web, HttpResponse};
use sqlx::SqlitePool;
use crate::{todo::model::TodoItem, utils};
use anyhow::{anyhow, Context};

#[tracing::instrument]
#[post("/todo")]
pub async fn todo_add(pool: web::Data<SqlitePool>, todo_item: web::Json<TodoItem>) -> Result<HttpResponse, utils::error::Error> {
    let mut conn = pool.acquire()
        .await
        .map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;
    
    let result = sqlx::query("insert into todos (description) values (?1)")
        .bind(todo_item.description.clone())
        .execute(&mut *conn)
        .await.map_err(|e| {
            utils::error::Error(anyhow!(e))
        })?;

    Ok(HttpResponse::Ok().body(result.last_insert_rowid().to_string()))
}