use serde_json::{json, Value};
use std::path::PathBuf;

use async_compression::tokio::bufread::GzipDecoder;
use tokio_tar::Archive;

use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio_stream::*;

use axum::body::Bytes;
use axum::response::Json;

use super::consts::*;
use super::error::*;

// FIXME: make this via parser, not that ad-hoc
/// replaces all relative links in markdown to links which will be pointing to static server adress
pub async fn edit_md_relative_urls(fd: String, url: String) {
    let mut content = String::new();
    {
        let mut file = File::open(&fd).await.unwrap();
        file.read_to_string(&mut content).await.unwrap();
        // FIXME: Scary scary thing
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

// TODO: JSON<VALUE> -> JSON<STRUCT>
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

// TODO: JSON<VALUE> TO JSON<RESP>
pub async fn handle_tar_gzip(article_name: &str, data: Bytes) -> Result<Json<Value>, ArticleError> {
    get_tar_gzip(article_name, data).await?;
    let _ = extract_tar_gzip(article_name).await?;
    Ok(Json(json!({"success": true, "data": "Ok"})))
}
