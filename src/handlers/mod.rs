use axum::response::Json;
use axum::routing::{get, Router};
use serde_json::{json, Value};

mod posts;

pub fn api_router() -> Router {
    Router::new().route("/", get(root))
}

pub async fn root() -> Json<Value> {
    Json(json!({ "status": 200, "data": "Hello, you are currently at / " }))
}
