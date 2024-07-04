use crate::AppState;
use axum::routing::{delete, get, post, put, Router};

mod error;
mod handler;
mod model;
mod response;
mod util;

mod consts {
    use lazy_static::lazy_static;
    use std::env;

    lazy_static! {
        pub(crate) static ref ARTICLES_PATH: String =
            env::var("ARTICLES").expect("No ARTICLES env var was found");
        pub(crate) static ref WEB_URL: String = format!(
            "{p}://{d}",
            p = env::var("PROTOCOL").expect("No PROTOCOL env var was found"),
            d = env::var("DOMAIN").expect("No DOMAIN env var was found")
        );
        pub(crate) static ref WEB_ARTICLES_URL: String = format!("{}/articles", *WEB_URL);
    }
}

pub fn api_router() -> Router<AppState> {
    use handler::*;
    Router::<AppState>::new()
        .route(
            "/",
            post(create_article).get(read_articles).put(update_article),
        )
        .route("/:title", get(read_article).delete(delete_article))
}
