use crate::{
    models::{Article, ArticleListPagination, CreateArticle, ObjectById, UpdateArticle},
    AppState,
};


use pulldown_cmark::{Event, Parser, Tag};
use tokio_stream::*;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tokio_tar::Archive;
use async_compression::tokio::{bufread::GzipDecoder};

use tokio::stream::*;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use axum::extract::{Path, Query, State, multipart, Multipart};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::query;
use std::path::PathBuf;
use std::env;

async fn edit_md_relative_urls(fd: String, url: String) {
    let mut content = String::new();
    {
        let mut file = File::open(&fd).await.unwrap();
        file.read_to_string(&mut content).await.unwrap();
        content = content.replace("](./imgs/", &format!("]({}imgs/", url));
    }
    let mut file = File::create(&fd).await.unwrap();
    file.write_all(&content.into_bytes()).await.unwrap();
    file.flush().await.unwrap();
}

pub async fn extract_tar_gzip(article_name: &str, content: &str) -> Result<Json<Value>, Json<Value>> {
    let article_name = article_name.replace(" ", "_");
    let path = PathBuf::from(format!("{env}/{article_name}/", env=env::var("ARTICLES").unwrap()));
    let mut path_name = path.clone();
    path_name.push(&article_name);

    let absolute_path = format!("{p}://{d}/articles/{t}/",
                          p=env::var("PROTOCOL").expect("no protocol in .env"),
                          d=env::var("DOMAIN").expect("no domain in .env"),
                          t=&article_name
    ); // FIXME: Move to const/static(lazy)?


    tokio::fs::create_dir_all(&path).await.expect("Failed to create dirs!");

    let decoded = STANDARD.decode(content);
    let mut archive = File::create(&path_name).await.unwrap();
    if let Ok(decoded) = decoded {
        tracing::info!("Writing data to archieve...");
        archive.write_all(&decoded).await.expect("failed to write_all()!");
    } else {
        tracing::error!("Decode error!");
        // FIXME: do something with this shitty error handling xddd
        return Err(Json(json!({"success": false, "data": "Unable to decode!"})));
    }

    tracing::info!("Unpacking article...");
    let probably_tar_gz = File::open(&path_name).await;
    if let Ok(tar_gz) = probably_tar_gz {
        let reader = BufReader::new(tar_gz);

        let tar = GzipDecoder::new(reader);
        let mut archive = Archive::new(tar).entries().unwrap();
        // archive.unpack(&path).await.expect("failed to unpack");
        // UNWRAP HELL AHAHHA
        while let Some(file) = archive.next().await {
            let mut file = file.unwrap();
            file.unpack_in(&path).await.unwrap();
        }
        tracing::info!("Successfully unpacked archive");
    } else {
        tracing::error!("Failed to unpack archive!");
        // FIXME: do something with this shitty error handling xddd
        return Err(Json(json!({"success": false, "data": "Unable to upload non-gzip file!"})));
    }
    Ok(Json(json!({"success": true, "data": "unpacked!"})))

}


/// Handler for creating
/// Request: Title, Base64 tar.gz
/// Response:  Json(Success/Not)
/// FIXME: Strange error handling and unwraps!
pub async fn create_article(
    State(pool): State<AppState>,
    Json(payload): Json<CreateArticle>,
) -> Result<Json<Value>, Json<Value>> {
    tracing::info!("Creating article with name: {}!", &payload.title);
    //FIXME: Need handle title is some
    // .replace(" ", "_") - everywhere, wtf??

    let _ = extract_tar_gzip(&payload.title, &payload.content).await?;

    let absolute_url = format!("{p}://{d}/articles/{t}/",
                          p=env::var("PROTOCOL").expect("no protocol in .env"),
                          d=env::var("DOMAIN").expect("no domain in .env"),
                          t=&payload.title.replace(" ", "_")
    ); // some doubts on this line...
    let md_path = format!("{}/main.md", absolute_url);
    let md_path_fs = format!("{env}/{article_name}/main.md", env=env::var("ARTICLES").unwrap(), article_name=&payload.title.replace(" ", "_"));
    println!("{}", md_path_fs);

    // let md_path_fs = PathBuf::from(format!("{env}/{article_name}/main.md", env=env::var("ARTICLES").unwrap()));
    edit_md_relative_urls(md_path_fs, absolute_url).await;

    let query = sqlx::query!(
        "INSERT INTO articles (title, content) values ($1, $2)",
        payload.title,
        md_path

    )
    .fetch_all(&pool.pool)
    .await;

    if let Err(e) = query {
        tracing::error!("Failed to create article, Error: {}", e);
        return Err(Json(json!({"success": false, "data": "Database issue!"})));
    }

    let response = json!({"success": true, "data": "Created!"});
    tracing::info!("Successfully uploaded article: {}", &payload.title);
    Ok(Json(response))
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

pub async fn read_article_exact(Path(title): Path<String>, State(pool): State<AppState>) -> Result<Json<Article>, Json<Value>> {
    tracing::info!("Reading exact article!");
    let query = sqlx::query_as!(
        Article,
        "SELECT * FROM articles WHERE title = $1",
        title
    )
    .fetch_one(&pool.pool)
    .await;

    match query {
        Ok(article) => Ok(Json(article)),
        Err(e) => {
            tracing::error!("Failed to get article from db!: {:?}", e);
            Err(Json(json!({"success": false, "data": "failed to get article!"})))
        }
    }
}


/// Handler for update
/// Optional: Content update, Title update or together
/// FIXME: What to do when we want to update?
/// BUG! TESTME!
pub async fn update_article(
    State(pool): State<AppState>,
    Json(payload): Json<UpdateArticle>,
) -> Result<Json<Value>, Json<Value>> {
    let query = match &payload.content {
        None => {
            sqlx::query!(
                "UPDATE articles SET title = $1 WHERE id = $2",
                payload.title,
                payload.id
            )
            .execute(&pool.pool)
            .await
        }
        Some(content) => {
            let _ = extract_tar_gzip(&payload.title, content).await?;
            sqlx::query!(
                "UPDATE articles SET title = $1 WHERE id = $2",
                payload.title,
                payload.id
            )
            .execute(&pool.pool)
            .await
        },
    };

    match query {
        Ok(_) => Ok(Json(json!({"success": true, "data": "Updated!"}))),
        Err(e) => Err(Json(
            json!({"success": false, "data": {"message" : "Failed to update!", "error": e.to_string()}}),
        )),
    }
}


pub async fn delete_article(
    Query(id): Query<ObjectById>,
    State(pool): State<AppState>,
) -> Result<Json<Value>, Json<Value>> {
    let query = match id.id {
        Some(idx) => {
            sqlx::query_as!(
                Article,
                "
                    DELETE FROM articles
                    WHERE id = $1
                ",
                idx
            )
            .execute(&pool.pool)
            .await
        }
        None => {
            return Err(Json(
                json!({"success": false, "data": {"error": "No index given!" }}),
            ));
        }
    };
    match query {
        Ok(_) => {
            //
            Ok(Json(
                json!({"success": true, "data": format!("deleted article :{}", id.id.unwrap())}),
            ))
        }
        Err(e) => {
            //
            Err(Json(
                json!({"success": false, "data": {"error": e.to_string(), "suggestion": "Failed to delete! I assume that such id doesn't exist"}}),
            ))
        }
    }
}


/*
async fn read_article() -> Response {
    todo!()
}

async fn update_article() -> Response {
    todo!()
}

async fn delete_article() -> Response {
    todo!()
}
*/
