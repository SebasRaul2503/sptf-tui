//! Composition root.
//!
//! Wires together: configuration, state, input, terminal, and the (future)
//! service layer. The [`App`] type owns the event loop and is the only place
//! where these layers meet.

mod app;
mod events;

pub use app::App;
pub use events::{AppEvent, EventSource};
