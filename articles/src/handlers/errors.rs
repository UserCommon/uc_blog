use std::fmt;

use axum::response::{IntoResponse, Json, Response};

use serde_json::json;

#[derive(Debug)]
pub enum ArticleError {
    Create(String),
    Read(String),
    Update(String),
    Delete(String),
}

impl fmt::Display for ArticleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Create(ref reason) => write!(f, "Creation error: failed to create. {}", reason),
            Self::Read(ref reason) => write!(f, "Reading error: failed to read. {}", reason),
            Self::Update(ref reason) => write!(f, "Updating error: failed to update. {}", reason),
            Self::Delete(ref reason) => write!(f, "Deleting error: failed to delete. {}", reason),
        }
    }
}

impl IntoResponse for ArticleError {
    fn into_response(self) -> Response {
        Json(json!({
            "success": false,
            "error": self.to_string()
        }))
        .into_response()
    }
}
