//! TODO: Result<Json, Json> -> Result<Model, Model>...
// TODO: ERRORS AS RESPONSES!
use crate::AppState;

use super::error::*;
use super::model::*;
use super::response::*;
use super::util::*;

use std::env;

use axum::extract::{Path, Query, State};
use axum::response::Json;
use axum_typed_multipart::TypedMultipart;

use serde_json::{json, Value};

use super::consts::*;

/// Handler for creating
/// Request: Title, tar.gz
/// Response:  Json(Success/Not)
/// SUGGESTIONS: This error may occur when directory exists
/// in fs. Because when im trynna create already existing article it's gives me this issue
pub async fn create_article(
    State(pool): State<AppState>,
    TypedMultipart(CreateArticleRequest { title, archive }): TypedMultipart<CreateArticleRequest>,
) -> Result<Json<CreateArticleResponse>, ArticleError> {
    let slug = &title.replace(' ', "_");
    tracing::info!("Creating article with title: {}!", &title);

    // check on existance
    let query = sqlx::query!("SELECT * FROM articles WHERE slug=$1", slug)
        .fetch_one(&pool.pool)
        .await;

    tracing::debug!("{:?}", query);
    if query.is_ok() {
        return Err(ArticleError::Create("Article already exist!".to_string()));
    }

    // Create article in fs
    let _ = handle_tar_gzip(slug, archive.contents).await?;

    let absolute_url = format!("{w}/{t}", w = *WEB_ARTICLES_URL, t = slug);
    let md_path = format!("{}/main.md", absolute_url);
    let md_path_fs = format!(
        "{env}/{article_name}/main.md",
        env = env::var("ARTICLES").unwrap(),
        article_name = slug
    );

    edit_md_relative_urls(md_path_fs, absolute_url).await;

    let query = sqlx::query!(
        "INSERT INTO articles (title, slug, content) values ($1, $2, $3)",
        title,
        slug,
        md_path
    )
    .fetch_all(&pool.pool)
    .await;

    if let Err(e) = query {
        tracing::error!("Failed to create article, Error: {}", e);
        return Err(ArticleError::Create("Database error".to_string()));
    }

    tracing::info!("Created article: {}", &title);
    Ok(Json(CreateArticleResponse {
        status: "success".to_string(),
        message: Some("created".to_string()),
        data: None,
    }))
}

/// Handler that implements read funcitonality
/// As long as articles stored in articles/ which is given in .env file
/// we can just return url to a filesystem in json which is showed for all
/// Response:
/// {
///     "title": "Article"
///     "content": "http:/.../articles/Article.md"
///     "created_at": 2024
/// }
pub async fn read_articles(
    Query(pagination): Query<ArticleListPagination>,
    State(pool): State<AppState>,
) -> Result<Json<ReadArticlesResponse>, ArticleError> {
    //              Uint getting negative value
    tracing::info!("Reading article!");
    let query = match pagination.page {
        Some(page) => {
            if page < 1 {
                return Err(ArticleError::Read("Page should be >= 1".into()));
            }
            let (s, e) = (page * 10 - 9, page * 10);
            sqlx::query_as!(
                Article,
                "SELECT * FROM articles WHERE id >= $1 AND id <= $2",
                s,
                e
            )
            .fetch_all(&pool.pool)
            .await
        }
        None => {
            sqlx::query_as!(Article, "SELECT * FROM articles")
                .fetch_all(&pool.pool)
                .await
        }
    };

    match query {
        Ok(data) => {
            tracing::info!("Read articles.");
            Ok(Json(ReadArticlesResponse {
                status: "success".into(),
                message: Some("Read articles!".into()),
                data: Some(data),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to read! {}", e);
            Err(ArticleError::Read("No such objects.".into()))
        }
    }
}

/// Handler for reading exact article
pub async fn read_article(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<ReadArticleResponse>, ArticleError> {
    let slug = title.replace(' ', "_"); // hard-coded FIXME:
    tracing::info!("Reading exact article!");
    let query = sqlx::query_as!(Article, "SELECT * FROM articles WHERE slug = $1", slug)
        .fetch_one(&pool.pool)
        .await;

    match query {
        Ok(data) => {
            tracing::info!("Read article. title: {}", title);
            Ok(Json(ReadArticleResponse {
                status: "success".into(),
                message: Some("Read article!".into()),
                data: Some(data),
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get article from db!: {:?}", e);
            Err(ArticleError::Read("Failed to read this article".into()))
        }
    }
}

/// Handler for update
/// Optional: Content update, Title update or together
/// hard coded trash
/// When i updating with the same tar.gz it's causes this error
/// FIXME: SLUG|TITLE is awful
pub async fn update_article(
    State(pool): State<AppState>,
    TypedMultipart(UpdateArticleRequest {
        title,
        new_title,
        archive,
    }): TypedMultipart<UpdateArticleRequest>,
) -> Result<Json<UpdateArticleResponse>, ArticleError> {
    let slug = title.replace(' ', "_").to_string();
    let updated_slug = new_title.clone().unwrap_or(title.clone()).replace(' ', "_");

    let article_dir_old = format!("{}/{}", *ARTICLES_PATH, slug);
    let article_dir_new = format!("{}/{}", *ARTICLES_PATH, updated_slug);

    // If user send new .tar.gz
    if let Some(ref content) = archive {
        // If we send content on update then it's
        // obvious that we had content before,
        // so it's okay to delete old files and
        // throw error if there no such files.
        delete_deprecated_files(&slug).await?;

        // Then we just extract our gzip!
        // .clone() should be zero cost?
        let _ = handle_tar_gzip(&slug, content.contents.clone()).await?;
    }
    if new_title.is_some() {
        println!("{}, {}", article_dir_old, article_dir_new);
        tokio::fs::rename(article_dir_old, article_dir_new)
            .await
            .expect("Failed to rename");
    }
    if let (None, None) = (archive, new_title.clone()) {
        return Err(ArticleError::Update("No parameters given.".into()));
    }

    let new_title = new_title.unwrap_or(title);

    let article_url_new = format!("{}/{}", *WEB_ARTICLES_URL, updated_slug);
    let content = format!("{}/main.md", article_url_new);
    let query = sqlx::query!(
        "UPDATE articles SET title = $1, content = $2, slug = $3 WHERE slug = $4",
        new_title,
        content,
        updated_slug,
        slug,
    )
    .execute(&pool.pool)
    .await;

    match query {
        Ok(_) => {
            tracing::info!("Updated article. new titile: {}", new_title);
            Ok(Json(UpdateArticleResponse {
                status: "success".into(),
                message: Some("Updated!".into()),
                data: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to execute query, error: {}", e);
            Err(ArticleError::Update("Failed to update article".into()))
        }
    }
}

/// FIXME: NOT DELETING CONTENT
pub async fn delete_article(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<DeleteArticleResponse>, ArticleError> {
    tracing::info!("Delete article. title: {}", &title);
    // FIXME: REMOVE ALL SPEC SYMBOLS!!
    let slug = title.replace(" ", "_");

    if let Err(e) = delete_deprecated_files(&slug).await {
        tracing::error!("Failed to delete files!, {}", e);
    }

    let query = sqlx::query_as!(
        Article,
        "
            DELETE FROM articles
            WHERE slug = $1
        ",
        slug
    )
    .execute(&pool.pool)
    .await;
    //json!({"success": true, "data": format!("deleted article :{}", &title)}),
    match query {
        Ok(_) => Ok(Json(DeleteArticleResponse {
            status: "success".into(),
            message: Some("Deleted article".into()),
            data: None,
        })),
        Err(e) => {
            tracing::error!("Failed to execute a query! error: {e}");
            Err(ArticleError::Delete("Failed to delete article".into()))
        }
    }
}
