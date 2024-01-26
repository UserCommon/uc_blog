use axum::ServiceExt;
use axum::{routing::get, Router};
use sqlx::sqlite::SqlitePool;
mod handlers;
use handlers::*;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().nest("/api", api_router());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
