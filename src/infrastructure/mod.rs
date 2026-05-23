//! Adapters to external systems: `DBus`, HTTP, filesystem.
//!
//! Implementations here implement traits defined in [`crate::services`] so the
//! rest of the application depends only on abstractions, not on concrete I/O.
//! Iteration 1 ships an empty surface; later iterations populate this module.
