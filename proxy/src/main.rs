use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::uri::Uri,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use axum_auth::AuthBasic;

use hyper::{Method, StatusCode};
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

mod auth {
    use lazy_static::lazy_static;
    lazy_static! {
        pub(crate) static ref USERNAME: String =
            std::env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME not set!");
        pub(crate) static ref PASSWORD: String =
            std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD not set!");
    }
}

mod cfg {
    use lazy_static::lazy_static;
    lazy_static! {
        pub(crate) static ref ARTICLES_DOMAIN: String =
            std::env::var("ARTICLES_DOMAIN").expect("ARTICLES_DOMAIN not set");
        pub(crate) static ref PROXY_DOMAIN: String =
            std::env::var("PROXY_DOMAIN").expect("PROXY_DOMAIN not set");
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let client: Client =
        hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
            .build(HttpConnector::new());

    let app = Router::new()
        .route(
            "/api/*path",
            get(handler).post(handler).put(handler).delete(handler),
        )
        .with_state(client);

    let listener = tokio::net::TcpListener::bind(&*cfg::PROXY_DOMAIN)
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// XXX: this code is mess
async fn handler(
    State(client): State<Client>,
    Path(url): Path<String>,
    auth: Option<AuthBasic>,
    mut req: Request,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let is_possible: bool = match *req.method() {
        Method::GET => true,
        Method::POST | Method::PUT | Method::DELETE => match auth {
            Some(AuthBasic((id, Some(passwd)))) => {
                id == *auth::USERNAME && passwd == *auth::PASSWORD
            }
            _ => false,
        },
        _ => false,
    };
    tracing::debug!("{}, {}, {}", is_possible, *req.method(), path_query);

    if is_possible == false {
        return Err(Json(
            serde_json::json!({"success": false, "error": "Not authenticated"}),
        ));
    }

    let uri = format!("http://{}/{}", *cfg::ARTICLES_DOMAIN, path_query);

    *req.uri_mut() = Uri::try_from(uri).unwrap();

    Ok(client
        .request(req)
        .await
        .map_err(|_| Json(serde_json::json!({"success": false, "error": "Failed on proxy"})))?
        .into_response())
}
