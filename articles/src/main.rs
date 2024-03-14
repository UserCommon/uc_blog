//! This programm allow you to have CRUD (create,read,update,delete)
//! functionality for articles via restful API
//! *There is no authentication* we consider you using gateway for this.
//!
//! ## Environment variables:
//! you must have, *DOMAIN*, *DATABASE_URL*, *ARTICLES*:
//! * DOMAIN - domain string
//! * DATABASE_URL - sqlite3://db/articles.db
//! * ARTICLES - is directory, where all articles stores
//!
//! ## ARCHITECTURE:
//! ```
//! Articles/
//!          Title/
//!               main.md
//!               imgs/
//! ```
//!  We need to send 1. Title
//!                  2. markdown
//!                  3. images (folder)
//! Send Article.tar.gz with all of these things.
//! **Important** that your markdown file must have main.md name
//! and all of your links in markdown has relative path!

use axum::{routing::get, Router, extract::Request, ServiceExt};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
mod utils;
mod handlers;
mod models;
use handlers::*;

#[derive(Clone)]
struct AppState {
    pub pool: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::EnvFilter::new(
    //         std::env::var("RUST_LOG").unwrap_or_else(|_| "axum_api=debug".into()),
    //     ))
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();
    tracing_subscriber::fmt::init();

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
    let articles_path = env::var("ARTICLES").expect("No ARTICLES env var");
    let serve_dir = ServeDir::new(&articles_path)
        .not_found_service(ServeFile::new(format!("{}/not_found.md", &articles_path)));
    let state = AppState { pool };
    let cors = CorsLayer::new().allow_origin(Any);
    // build our application with a single route
    let app = Router::new()
        .layer(cors)
        .nest_service("/articles", serve_dir.clone())
        .nest("/api", api_router())
        .with_state(state);
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("Listening on {}", "localhost");
    axum::serve(listener, ServiceExt::<Request>::into_make_service(app)).await.unwrap();
}
