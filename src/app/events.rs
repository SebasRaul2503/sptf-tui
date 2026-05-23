//! The event abstraction the [`super::App`] loop selects on.
//!
//! Folding terminal input, ticks, and (later) MPRIS notifications into a
//! single enum keeps the main loop a single `select!` rather than a tangle of
//! interleaved branches.

use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time;

/// All events the app loop can react to.
#[derive(Debug)]
pub enum AppEvent {
    /// Periodic redraw / position-poll tick.
    Tick,
    /// Raw terminal input.
    Input(CrosstermEvent),
    /// Underlying event source produced an error; surfaced for UI display.
    Error(String),
}

/// Pumps terminal + tick events into a channel.
///
/// The channel-based design lets us add MPRIS-originated events later (from a
/// different task) without restructuring the main loop.
pub struct EventSource {
    rx: mpsc::Receiver<AppEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl EventSource {
    /// Spawn the background pump.
    pub fn spawn(frame_rate: u16) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let tick_interval = Duration::from_millis(1_000 / u64::from(frame_rate.clamp(1, 240)));

        let handle = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut ticker = time::interval(tick_interval);
            ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            loop {
                let event = tokio::select! {
                    _ = ticker.tick() => AppEvent::Tick,
                    maybe = reader.next() => match maybe {
                        Some(Ok(ev)) => AppEvent::Input(ev),
                        Some(Err(err)) => AppEvent::Error(err.to_string()),
                        None => break,
                    },
                };

                if tx.send(event).await.is_err() {
                    break; // receiver dropped — app is shutting down.
                }
            }
        });

        Self { rx, _handle: handle }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
