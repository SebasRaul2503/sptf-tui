//! In-memory album-art cache.
//!
//! Keyed by the original art URL. Holds [`DynamicImage`]s behind an [`Arc`]
//! so cache hits are cheap and so the loader task can ship the image to the
//! main thread without copying the pixel buffer.
//!
//! Iteration 8 polish will add a bounded LRU + on-disk persistence; this
//! iteration keeps the cache unbounded but small (one entry per track played
//! during a session) which is fine for typical use.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use image::DynamicImage;

#[derive(Default)]
pub struct ArtCache {
    inner: RwLock<HashMap<String, Arc<DynamicImage>>>,
}

impl ArtCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, url: &str) -> Option<Arc<DynamicImage>> {
        self.inner.read().ok()?.get(url).cloned()
    }

    pub fn put(&self, url: String, image: Arc<DynamicImage>) {
        if let Ok(mut guard) = self.inner.write() {
            guard.insert(url, image);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().map_or(0, |g| g.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbImage};

    fn tiny_image() -> Arc<DynamicImage> {
        Arc::new(DynamicImage::ImageRgb8(RgbImage::new(2, 2)))
    }

    #[test]
    fn get_returns_none_for_missing_url() {
        let cache = ArtCache::new();
        assert!(cache.get("https://x").is_none());
        assert!(cache.is_empty());
    }

    #[test]
    fn put_then_get_round_trips() {
        let cache = ArtCache::new();
        cache.put("https://x".to_string(), tiny_image());
        assert!(cache.get("https://x").is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn put_overwrites_existing_entry() {
        let cache = ArtCache::new();
        cache.put("u".into(), tiny_image());
        cache.put("u".into(), tiny_image());
        assert_eq!(cache.len(), 1);
    }
}
