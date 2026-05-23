//! Bounded album-art LRU cache.
//!
//! Keyed by the original art URL. Holds [`DynamicImage`]s behind an [`Arc`]
//! so cache hits are cheap and the loader task can ship the image to the
//! main thread without copying the pixel buffer.
//!
//! The cache has a fixed capacity (default [`DEFAULT_CAPACITY`]) so a long
//! listening session can't grow it without bound. Eviction is least-recently-
//! used: the oldest unused entry is dropped when a new one is inserted into
//! a full cache.

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use image::DynamicImage;
use lru::LruCache;

pub const DEFAULT_CAPACITY: usize = 32;

pub struct ArtCache {
    inner: Mutex<LruCache<String, Arc<DynamicImage>>>,
}

impl ArtCache {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity.max(1)).expect("clamped to >=1");
        Self { inner: Mutex::new(LruCache::new(cap)) }
    }

    pub fn get(&self, url: &str) -> Option<Arc<DynamicImage>> {
        self.inner.lock().ok()?.get(url).cloned()
    }

    pub fn put(&self, url: String, image: Arc<DynamicImage>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.put(url, image);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().map_or(0, |g| g.len())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ArtCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, RgbImage};

    use super::*;

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

    #[test]
    fn lru_evicts_oldest_when_full() {
        let cache = ArtCache::with_capacity(2);
        cache.put("a".into(), tiny_image());
        cache.put("b".into(), tiny_image());
        // touching "a" makes it most-recent
        let _ = cache.get("a");
        cache.put("c".into(), tiny_image());
        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_none());
        assert!(cache.get("c").is_some());
    }
}
