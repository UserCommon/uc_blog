use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromMultipart};
use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::FromRow;

/// Content is url
#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(TryFromMultipart)]
pub struct CreateArticleRequest {
    pub title: String,
    pub archive: FieldData<Bytes>,
}

#[derive(TryFromMultipart)]
pub struct UpdateArticleRequest {
    pub title: String,
    pub new_title: Option<String>,
    pub archive: Option<FieldData<Bytes>>,
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
