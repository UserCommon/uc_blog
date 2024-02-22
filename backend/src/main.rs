use axum::ServiceExt;
use axum::{routing::get, Router};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;
use handlers::*;

#[derive(Clone)]
struct AppState {
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "axum_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations!");

    let state = AppState { pool };
    let cors = CorsLayer::new().allow_origin(Any);
    // build our application with a single route
    let app = Router::new()
        .layer(cors)
        .nest("/api", api_router())
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Listening on {}", "localhost");
    axum::serve(listener, app).await.unwrap();
}
