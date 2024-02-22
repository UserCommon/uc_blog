use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateArticle {
    pub title: String,
    pub content: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateArticle {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author: String,
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
