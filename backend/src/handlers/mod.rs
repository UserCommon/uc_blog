use axum::extract::State;
use axum::response::Json;
use axum::routing::{delete, get, post, put, Router};
use serde_json::{json, Value};

use crate::AppState;

mod articles;

pub fn api_router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/", get(root))
        .route("/create", post(articles::create_article))
        .route("/get", get(articles::read_article_list))
        .route("/get/:title", get(articles::read_article_exact))
        .route("/delete", delete(articles::delete_article))
        .route("/update", put(articles::update_article))
}

pub async fn root() -> Json<Value> {
    Json(json!({ "status": 200, "data": "Hello, you are currently at / " }))
}
