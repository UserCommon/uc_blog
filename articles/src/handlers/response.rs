use serde::{Deserialize, Serialize};

use super::model::*;

// `CreateArticleResponse` used on article create handler
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateArticleResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Article>,
}

/// `ReadArticleResponse` used on article read handler
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadArticleResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Article>,
}

/// `ReadArticlesResponse` used on articles read handler
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadArticlesResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Vec<Article>>,
}

/// `UpdateArticleResponse` used on article update handler
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateArticleResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Article>,
}

/// `DeleteArticleResponse` used on article delete handler
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteArticleResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Article>,
}
