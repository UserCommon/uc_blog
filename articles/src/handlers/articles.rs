//! TODO: Result<Json, Json> -> Result<Model, Model>...

use crate::{
    models::{
        Article, ArticleListPagination, CreateArticle, ObjectById, ObjectByTitle, UpdateArticle, UploadArticleRequest, UpdateArticleRequest,
    },
    AppState,
};

use axum::body::{ Bytes, to_bytes };
use async_compression::tokio::bufread::GzipDecoder;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use pulldown_cmark::{Event, Parser, Tag};
use tokio_stream::*;
use tokio_tar::Archive;
use axum_typed_multipart::TypedMultipart;
use super::errors;
use axum::extract::{multipart, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, Router};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::query;
use std::{env};
use std::path::PathBuf;
use tokio::fs::{ File, OpenOptions };
use tokio::io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::stream::*;

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
pub async fn delete_deprecated_files<T>(article_old_name: &T) -> Result<(), errors::ArticleFsError>
where T: AsRef<str> + ?Sized
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
            Err(errors::ArticleFsError::FailedToDelete)
        }
    }
}


/// Gets .tar.gz binary and writes it to server
pub async fn get_tar_gzip<T>(article_name: &T, data: Bytes) -> Result<(), Json<Value>>
where T: AsRef<str> + ?Sized
{
    let article_name = article_name.as_ref();

    let path = PathBuf::from(format!("{env}/{article_name}/", env = *ARTICLES_PATH));
    let mut path_name = path.clone();
    path_name.push(&article_name);

    tokio::fs::create_dir_all(path)
        .await
        .expect("Failed to create dirs!");

    let mut archive = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path_name).await.unwrap();
    // TODO: checksums?
    tracing::info!("Writing data to archieve...");
    // Corruption may occure there???
    archive
        .write_all(&data)
        .await
        .unwrap();

    archive.flush().await.unwrap();
    Ok(())
}

pub async fn extract_tar_gzip<T>(
    article_name: &T,
) -> Result<Json<Value>, Json<Value>>
where T: AsRef<str> + ?Sized
{
    let article_name = article_name.as_ref();
    let path = PathBuf::from(format!("{env}/{article_name}/", env = *ARTICLES_PATH));
    let mut path_name = path.clone();
    path_name.push(&article_name);

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

pub async fn handle_tar_gzip(article_name: &str, data: Bytes) -> Result<Json<Value>, Json<Value>> {
    let _ = get_tar_gzip(article_name, data).await?;
    let _ = extract_tar_gzip(article_name).await?;
    Ok(Json(json!({"success": true, "data": "Ok"})))
}

/// Handler for creating
/// Request: Title, Base64 tar.gz
/// Response:  Json(Success/Not)
/// FIXME: Strange error handling and unwraps!
/// SUGGESTIONS: This error may occur when directory exists
/// in fs. Because when im trynna create already existing article it's gives me this issue
pub async fn create_article(
    State(pool): State<AppState>,
    TypedMultipart(UploadArticleRequest { title, archive }): TypedMultipart<UploadArticleRequest>
) -> Result<Json<Value>, Json<Value>> {

    let title = &title.replace(" ", "_");

    tracing::debug!("{}", title);

    // check on existance
    let query = sqlx::query!(
        "SELECT * FROM articles WHERE title=$1",
        title
    )
        .fetch_one(&pool.pool)
        .await;

    tracing::debug!("{:?}",query);
    if let Ok(_) = query {
        return Err(Json(json!({"success": false, "data": "Article already exist!"})));
    }

    tracing::info!("Creating article with name: {}!", &title);
    //FIXME: Need handle title is some
    // .replace(" ", "_") - everywhere, wtf??

    let _ = handle_tar_gzip(title, archive.contents).await?;

    let absolute_url = format!("{w}/{t}", w = *WEB_ARTICLES_URL, t = title);
    let md_path = format!("{}/main.md", absolute_url);
    let md_path_fs = format!(
        "{env}/{article_name}/main.md",
        env = env::var("ARTICLES").unwrap(),
        article_name = title
    );
    println!("{}", md_path_fs);

    // let md_path_fs = PathBuf::from(format!("{env}/{article_name}/main.md", env=env::var("ARTICLES").unwrap()));
    edit_md_relative_urls(md_path_fs, absolute_url).await;

    let query = sqlx::query!(
        "INSERT INTO articles (title, content) values ($1, $2)",
        title,
        md_path
    )
    .fetch_all(&pool.pool)
    .await;

    if let Err(e) = query {
        tracing::error!("Failed to create article, Error: {}", e);
        return Err(Json(json!({"success": false, "data": "Database issue!"})));
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
) -> Result<Json<Vec<Article>>, Json<Value>> {
    //              Uint getting negative value
    tracing::info!("Reading article!");
    let query = match pagination.page {
        Some(page) => {
            if page < 1 {
                return Err(Json(
                    json!({"success": false, "error": "page should be >= 1"}),
                ));
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
            let e_str = e.to_string();
            tracing::error!("Failed to read! {}", e_str);
            let resp = json!({"success": false, "data": {"error": e_str}});
            Err(Json(resp))
        }
    }
}

/// Handler for reading exact article
pub async fn read_article_exact(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<Article>, Json<Value>> {
    let title = title.replace(" ", "_"); // hard-coded
    tracing::info!("Reading exact article!");
    let query = sqlx::query_as!(Article, "SELECT * FROM articles WHERE title = $1", title)
        .fetch_one(&pool.pool)
        .await;

    match query {
        Ok(article) => Ok(Json(article)),
        Err(e) => {
            tracing::error!("Failed to get article from db!: {:?}", e);
            Err(Json(
                json!({"success": false, "data": "failed to get article!"}),
            ))
        }
    }
}

/// Handler for update
/// Optional: Content update, Title update or together
/// FIXME: What to do when we want to update?
/// BUG! TESTME!
/// hard coded trash
/// When i updating with the same tar.gz it's causes this error
pub async fn update_article(
    State(pool): State<AppState>,
    TypedMultipart(UpdateArticleRequest { title, new_title, archive }): TypedMultipart<UpdateArticleRequest>
) -> Result<Json<Value>, Json<Value>> {
    let title = title.replace(" ", "_").to_string();
    let updated_title = new_title.clone().unwrap_or(title.clone()).replace(" ", "_");

    let article_dir_old = format!("{}/{}", *ARTICLES_PATH, title);
    let article_dir_new = format!("{}/{}", *ARTICLES_PATH, updated_title);

    // FIXME: spooky if elsing. Trash.
    // If user scend new .tar.gz
    if let Some(content) = archive {
        // If we send content on update then it's
        // obvious that we had content before,
        // so it's okay to delete old files and
        // throw error if there no such files.
        if let Ok(_) = delete_deprecated_files(&title).await {
            tracing::info!("Deleted deprecated files");
        } else {
            tracing::error!("Failed to deleted deprecated files");
            return Err(Json(
                json!({"success": false, "data": "Unable to delete old files. Probably because there is no such article?"}),
            ));
        }

        // Then we just extract our gzip!
        let _ = handle_tar_gzip(&updated_title, content.contents).await?;
    } else {
        // Else if there only tile been send we can just update
        // directory name and db record!
        if let Some(_) = new_title {
                        println!("{}, {}", article_dir_old, article_dir_new);
            // FIXME: article_dir_new already exists => Fail
            tokio::fs::rename(article_dir_old, article_dir_new)
                .await
                .expect("Failed to rename");
        } else {
            return Err(Json(json!({"success": false, "data": "No parameters!"})));
        }
    }


    let article_url_new = format!("{}/{}", *WEB_ARTICLES_URL, updated_title);
    let content = format!("{}/main.md", article_url_new);
    let query = sqlx::query!(
        "UPDATE articles SET title = $1, content = $2 WHERE title = $3",
        updated_title,
        content,
        title
    )
        .execute(&pool.pool)
        .await;

    match query {
        Ok(_) => Ok(Json(json!({"success": true, "data": "Updated!"}))),
        Err(e) => {
            tracing::error!("Failed to execute query, error: {e}");
            Err(Json(
                json!({"success": false, "data": "Failed to update article!"}),
            ))
        }
    }
}

/// FIXME: NOT DELETING CONTENT
pub async fn delete_article(
    Path(title): Path<String>,
    State(pool): State<AppState>,
) -> Result<Json<Value>, Json<Value>> {
    let title = title.replace(" ", "_");

    let del_res = delete_deprecated_files(&title).await;
    if let Err(e) = del_res {
        tracing::error!(
            "Failed to delete files for: {t} with error: {e}",
            t = &title
        );
        return Err(Json(
            json!({"success": false, "data": "Failed to delete article!"}),
        ));
    }

    let query = sqlx::query_as!(
        Article,
        "
            DELETE FROM articles
            WHERE title = $1
        ",
        title
    )
    .execute(&pool.pool)
    .await;

    match query {
        Ok(_) => Ok(Json(
            json!({"success": true, "data": format!("deleted article :{}", &title)}),
        )),
        Err(e) => {
            tracing::error!("Failed to execute a query! error: {e}");
            Err(Json(
                json!({"success": false, "data": "Failed to delete article!"}),
            ))
        }
    }
}