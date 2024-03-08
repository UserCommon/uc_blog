use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Content is base64 binary
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateArticle {
    pub title: String,
    pub archive: Vec<u8>,
}

/// Content is base64 binary
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateArticle {
    pub title: String,
    pub new_title: Option<String>,
    pub archive: Option<Vec<u8>>,
}

/// Content is url
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct ArticleScheme {
    pub id: i64,
    pub title: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ArticleListPagination {
    pub page: Option<u32>,
    // pub per_page: u32, maybe later this will be helpful...
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ObjectById {
    pub id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectByTitle {
    pub title: String,
}
