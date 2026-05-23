//! Lazy initialization of [`ratatui_image::picker::Picker`].
//!
//! The picker queries the terminal for graphics-protocol support and font
//! size. Both queries write to *and* read from the terminal, so they must be
//! done *before* the application enters the alternate screen / raw mode.
//! A failure (terminals that don't reply, dumb pipes in CI, …) falls back to
//! a sensible cell size; ratatui-image will then pick Halfblocks rendering.

use ratatui_image::picker::Picker;
use tracing::{debug, warn};

const FALLBACK_FONT_SIZE: (u16, u16) = (8, 16);

/// Build a [`Picker`] for the current terminal. Never fails: a fallback
/// picker is used when the terminal cannot be queried (e.g. piped output).
pub fn init_picker() -> Picker {
    match Picker::from_query_stdio() {
        Ok(picker) => {
            debug!(
                protocol = ?picker.protocol_type(),
                font_size = ?picker.font_size(),
                "picker initialized from stdio query"
            );
            picker
        }
        Err(err) => {
            warn!(?err, "stdio picker query failed, falling back to halfblocks");
            Picker::from_fontsize(FALLBACK_FONT_SIZE)
        }
    }
}
