//! Album-cover rendering pipeline.
//!
//! The flow:
//!
//! ```text
//! PlayerSnapshot.art_url ─▶ loader::fetch ─▶ DynamicImage
//!                              ▲                  │
//!                              │            cache::ArtCache (Arc<DynamicImage>)
//!                              │                  ▼
//!                          picker.new_resize_protocol(image)
//!                                       │
//!                                       ▼
//!                        AppState.art (StatefulProtocol)
//!                                       │
//!                                       ▼
//!                          widgets::album_art::render
//! ```
//!
//! Protocol selection (Kitty, iTerm2, Sixel, Halfblocks fallback) is handled
//! by [`ratatui_image::picker::Picker`] under the hood; the only failure
//! mode the rest of the app cares about is "no usable protocol" — which
//! ratatui-image avoids by falling back to halfblocks, so we always render
//! *something*.

pub mod cache;
pub mod loader;
pub mod picker;

pub use cache::ArtCache;
pub use loader::fetch_image;
pub use picker::init_picker;
