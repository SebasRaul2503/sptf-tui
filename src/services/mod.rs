//! Service-layer traits.
//!
//! Defines the abstractions that the UI and `App` depend on (player control,
//! album-art fetching, …). Concrete implementations live in
//! [`crate::infrastructure`]; tests can supply in-memory fakes.

pub mod player;

pub use player::PlayerService;
