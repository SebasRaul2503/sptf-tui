//! Reusable widget primitives.
//!
//! Iteration 1 keeps this module deliberately empty — the only "view" code
//! currently lives in [`crate::tui::view`] and is small enough to inline.
//! Later iterations split widgets (progress bar, album cover, controls bar)
//! out so they can be unit-tested in isolation against a `TestBackend`.
