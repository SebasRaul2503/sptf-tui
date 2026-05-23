//! Asynchronous image loader.
//!
//! Supports `http(s)://` (via reqwest) and `file://` schemes. Image decoding
//! is offloaded to `tokio::task::spawn_blocking` because the `image` crate is
//! CPU-bound and would otherwise stall the runtime.

use std::sync::Arc;

use image::DynamicImage;
use tokio::task;
use url::Url;

use crate::core::error::ArtError;

/// Fetch and decode an image from any supported URL scheme.
pub async fn fetch_image(url: &str) -> Result<Arc<DynamicImage>, ArtError> {
    if url.is_empty() {
        return Err(ArtError::Unavailable);
    }

    let parsed = Url::parse(url).map_err(|e| ArtError::Network(e.to_string()))?;

    let bytes = match parsed.scheme() {
        "file" => read_file(&parsed).await?,
        "http" | "https" => fetch_http(url).await?,
        other => return Err(ArtError::Network(format!("unsupported scheme {other:?}"))),
    };

    let image = decode_bytes(bytes).await?;
    Ok(Arc::new(image))
}

async fn read_file(parsed: &Url) -> Result<Vec<u8>, ArtError> {
    let path =
        parsed.to_file_path().map_err(|()| ArtError::Network("invalid file:// URL".to_string()))?;
    tokio::fs::read(&path).await.map_err(|e| ArtError::Network(e.to_string()))
}

async fn fetch_http(url: &str) -> Result<Vec<u8>, ArtError> {
    let response = reqwest::get(url).await.map_err(|e| ArtError::Network(e.to_string()))?;
    let response = response.error_for_status().map_err(|e| ArtError::Network(e.to_string()))?;
    let bytes = response.bytes().await.map_err(|e| ArtError::Network(e.to_string()))?;
    Ok(bytes.to_vec())
}

async fn decode_bytes(bytes: Vec<u8>) -> Result<DynamicImage, ArtError> {
    task::spawn_blocking(move || image::load_from_memory(&bytes))
        .await
        .map_err(|e| ArtError::Decode(e.to_string()))?
        .map_err(|e| ArtError::Decode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_url_is_unavailable() {
        assert!(matches!(fetch_image("").await, Err(ArtError::Unavailable)));
    }

    #[tokio::test]
    async fn unsupported_scheme_errors() {
        let err = fetch_image("ftp://example.com/x.png").await.unwrap_err();
        assert!(matches!(err, ArtError::Network(_)));
    }

    #[tokio::test]
    async fn invalid_url_errors() {
        let err = fetch_image("not a url").await.unwrap_err();
        assert!(matches!(err, ArtError::Network(_)));
    }
}
