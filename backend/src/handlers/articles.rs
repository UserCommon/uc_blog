use crate::{
    models::{Article, ArticleListPagination, CreateArticle, ObjectById, UpdateArticle},
    AppState,
};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::query;

pub async fn create_article(
    State(pool): State<AppState>,
    Json(payload): Json<CreateArticle>,
) -> Json<Value> {
    let query = sqlx::query!(
        "INSERT INTO articles (title, content, author) values ($1, $2, $3)",
        payload.title,
        payload.content,
        payload.author
    )
    .fetch_all(&pool.pool)
    .await
    .expect("failed to create!"); // TODO! Remake pls!

    let response = json!({"success": true, "data": "Created!"});
    Json(response)
}

pub async fn read_article(
    Query(pagination): Query<ArticleListPagination>,
    State(pool): State<AppState>,
) -> Result<Json<Vec<Article>>, Json<Value>> {
    //              Uint getting negative value
    let query = match pagination.page {
        Some(page) => {
            if page < 1 {
                return Err(Json(
                    json!({"success": "failed", "error": "page should be >= 1"}),
                ));
            }
            let (s, e) = (page * 10 - 9, page * 10);
            sqlx::query_as!(
                Article,
                "SELECT * FROM articles WHERE id >= $1 AND id <= $2",
                s,
                e
            )
            .fetch_all(&pool.pool)
            .await
        }
        None => {
            sqlx::query_as!(Article, "SELECT * FROM articles")
                .fetch_all(&pool.pool)
                .await
        }
    };

    match query {
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            let e_str = e.to_string();
            let resp = json!({"success": false, "data": {"error": e_str}});
            Err(Json(resp))
        }
    }
}

pub async fn delete_article(
    Query(id): Query<ObjectById>,
    State(pool): State<AppState>,
) -> Result<Json<Value>, Json<Value>> {
    let query = match id.id {
        Some(idx) => {
            sqlx::query_as!(
                Article,
                "
                    DELETE FROM articles
                    WHERE id = $1
                ",
                idx
            )
            .execute(&pool.pool)
            .await
        }
        None => {
            return Err(Json(
                json!({"success": false, "data": {"error": "No index given!" }}),
            ));
        }
    };
    match query {
        Ok(_) => {
            //
            Ok(Json(
                json!({"success": true, "data": format!("deleted article :{}", id.id.unwrap())}),
            ))
        }
        Err(e) => {
            //
            Err(Json(
                json!({"success": false, "data": {"error": e.to_string(), "suggestion": "Failed to delete! I assume that such id doesn't exist"}}),
            ))
        }
    }
}

pub async fn update_article(
    State(pool): State<AppState>,
    Json(payload): Json<UpdateArticle>,
) -> Result<Json<Value>, Json<Value>> {
    let query = match (payload.title.clone(), payload.content.clone()) {
        (Some(title), Some(content)) => {
            sqlx::query!(
                "UPDATE articles SET title = $1, content = $2 WHERE id = $3",
                title,
                content,
                payload.id
            )
            .execute(&pool.pool)
            .await
        }
        (Some(title), None) => {
            sqlx::query!(
                "UPDATE articles SET title = $1 WHERE id = $2",
                title,
                payload.id
            )
            .execute(&pool.pool)
            .await
        }
        (None, Some(content)) => {
            sqlx::query!(
                "UPDATE articles SET content = $1 WHERE id = $2",
                content,
                payload.id
            )
            .execute(&pool.pool)
            .await
        }
        (None, None) => {
            return Err(Json(
                json!({"success": false, "data": "No parameters for updating given!"}),
            ));
        }
    };

    match query {
        Ok(_) => Ok(Json(json!({"success": true, "data": "Updated!"}))),
        Err(e) => Err(Json(
            json!({"success": false, "data": {"message" : "Failed to update!", "error": e.to_string()}}),
        )),
    }
}

/*
async fn read_article() -> Response {
    todo!()
}

async fn update_article() -> Response {
    todo!()
}

async fn delete_article() -> Response {
    todo!()
}
*/
