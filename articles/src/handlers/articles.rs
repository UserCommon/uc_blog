//! TODO: Result<Json, Json> -> Result<Model, Model>...

use crate::AppState;

use super::models::{Article, ArticleListPagination, UpdateArticleRequest, UploadArticleRequest};

use super::errors::*;

use async_compression::tokio::bufread::GzipDecoder;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::response::Json;
use axum_typed_multipart::TypedMultipart;
use tokio_stream::*;
use tokio_tar::Archive;

use lazy_static::lazy_static;

use serde_json::{json, Value};

use std::env;
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

lazy_static! {
    static ref ARTICLES_PATH: String = env::var("ARTICLES").expect("No ARTICLES env var was found");
    static ref WEB_URL: String = format!(
        "{p}://{d}",
        p = env::var("PROTOCOL").expect("No PROTOCOL env var was found"),
        d = env::var("DOMAIN").expect("No DOMAIN env var was found")
    );
    static ref WEB_ARTICLES_URL: String = format!("{}/articles", *WEB_URL);
}

async fn edit_md_relative_urls(fd: String, url: String) {
    let mut content = String::new();
    {
        let mut file = File::open(&fd).await.unwrap();
        file.read_to_string(&mut content).await.unwrap();
        content = content.replace("](./imgs/", &format!("]({}imgs/", url));
    }
    let mut file = File::create(&fd).await.unwrap();
    file.write_all(&content.into_bytes()).await.unwrap();
    file.flush().await.unwrap(); // WHAT THE FUCK!!!! I SPENT 10 HOURS BECAUSE OF THIS FUCKING LINE!!! AHAHAHAHH
}

/// XXX: DANGEROUS CODE!
pub async fn delete_deprecated_files<T>(article_old_name: &T) -> Result<(), ArticleError>
where
    T: AsRef<str> + ?Sized,
{
    let article_old_name = article_old_name.as_ref();
    let path = format!("{}/{}", *ARTICLES_PATH, article_old_name);
    let res = tokio::fs::remove_dir_all(&path).await;

    match res {
        Ok(_) => {
            tracing::info!("Deleted {}", article_old_name);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to delete {} with error: {}", &path, e);
            Err(ArticleError::Delete(
                "Failed to delete deprecated files!".into(),
            ))
        }
    }
}

/// Gets .tar.gz binary and writes it to server
pub async fn get_tar_gzip<T>(article_name: &T, data: Bytes) -> Result<(), ArticleError>
where
    T: AsRef<str> + ?Sized,
{
    let article_name = article_name.as_ref();

    let path = PathBuf::from(format!("{env}/{article_name}/", env = *ARTICLES_PATH));
    let mut path_name = path.clone();
    path_name.push(article_name);

    if let Err(e) = tokio::fs::create_dir_all(path).await {
        tracing::error!("failed to create dir in *get_tar_gzip*: {}", e);
        return Err(ArticleError::Create("Server issue".into()));
    }

    let archive_unchecked = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path_name)
        .await;

    let mut archive = match archive_unchecked {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to write into archive dir in *get_tar_gzip*: {}", e);
            return Err(ArticleError::Create("Server issue".into()));
        }
    };

    tracing::info!("Writing data to archieve...");
    if let Err(e) = archive.write_all(&data).await {
        tracing::error!("failed to write into archive dir in *get_tar_gzip*: {}", e);
        return Err(ArticleError::Create("Server issue".into()));
    }

    archive.flush().await.unwrap();
    Ok(())
}

pub async fn extract_tar_gzip<T>(article_name: &T) -> Result<Json<Value>, ArticleError>
where
    T: AsRef<str> + ?Sized,
{
    let article_name = article_name.as_ref();
    let path = PathBuf::from(format!("{env}/{article_name}/", env = *ARTICLES_PATH));
    let mut path_name = path.clone();
    path_name.push(article_name);

    tracing::info!("Extracting article...");
    let s = format!("{}/{article_name}", path.to_str().unwrap());
    tracing::debug!("{}", &s);
    let tar_gz = File::open(&s).await.unwrap();

    let reader = BufReader::new(tar_gz);

    let tar = GzipDecoder::new(reader);
    let mut archive = Archive::new(tar).entries().unwrap();

    while let Some(file) = archive.next().await {
        let mut file = file.unwrap();
        file.unpack_in(&path).await.unwrap();
    }
    tracing::info!("Successfully unpacked archive");

    Ok(Json(json!({"success": true, "data": "unpacked!"})))
}

pub async fn handle_tar_gzip(article_name: &str, data: Bytes) -> Result<Json<Value>, ArticleError> {
    get_tar_gzip(article_name, data).await?;
    let _ = extract_tar_gzip(article_name).await?;
    Ok(Json(json!({"success": true, "data": "Ok"})))
}

/// Handler for creating
/// Request: Title, tar.gz
/// Response:  Json(Success/Not)
/// SUGGESTIONS: This error may occur when directory exists
/// in fs. Because when im trynna create already existing article it's gives me this issue
pub async fn create_article(
    State(pool): State<AppState>,
    TypedMultipart(UploadArticleRequest { title, archive }): TypedMultipart<UploadArticleRequest>,
) -> Result<Json<Value>, ArticleError> {
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

    tracing::info!("Successfully uploaded article: {}", &title);
    Ok(Json(json!({"success": true, "data": "Created!"})))
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
pub async fn read_article_list(
    Query(pagination): Query<ArticleListPagination>,
    State(pool): State<AppState>,
) -> Result<Json<Vec<Article>>, ArticleError> {
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
        Ok(v) => Ok(Json(v)),
        Err(e) => {
            tracing::error!("Failed to read! {}", e);
            Err(ArticleError::Read("No such objects.".into()))
        }
    }
}

/// Handler for reading exact article
pub async fn read_article_exact(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<Article>, ArticleError> {
    let slug = title.replace(' ', "_"); // hard-coded
    tracing::info!("Reading exact article!");
    let query = sqlx::query_as!(Article, "SELECT * FROM articles WHERE slug = $1", slug)
        .fetch_one(&pool.pool)
        .await;

    match query {
        Ok(article) => Ok(Json(article)),
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
) -> Result<Json<Value>, ArticleError> {
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
        Ok(_) => Ok(Json(json!({"success": true, "data": "Updated!"}))),
        Err(e) => {
            tracing::error!("Failed to execute query, error: {e}");
            Err(ArticleError::Update("Failed to update article".into()))
        }
    }
}

/// FIXME: NOT DELETING CONTENT
pub async fn delete_article(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<Value>, ArticleError> {
    let slug = title.replace(' ', "_");

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

    match query {
        Ok(_) => Ok(Json(
            json!({"success": true, "data": format!("deleted article :{}", &title)}),
        )),
        Err(e) => {
            tracing::error!("Failed to execute a query! error: {e}");
            Err(ArticleError::Delete("Failed to delete article".into()))
        }
    }
}
