//! Application event channel.
//!
//! Folding terminal input, ticks, and (later) MPRIS notifications into a
//! single enum keeps the main loop a single `select!` rather than a tangle of
//! interleaved branches. Every producer is a `tokio::spawn`ed task that owns
//! a clone of a single [`mpsc::Sender`]; the main loop owns the only
//! [`mpsc::Receiver`].

use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::debug;

use std::sync::Arc;

use image::DynamicImage;

use crate::core::error::{ArtError, PlayerError};
use crate::domain::PlayerSnapshot;

/// All events the app loop can react to.
#[derive(Debug)]
pub enum AppEvent {
    /// Periodic redraw tick.
    Tick,
    /// Raw terminal input.
    Input(CrosstermEvent),
    /// Snapshot pushed by the MPRIS watcher.
    PlayerSnapshot(Box<PlayerSnapshot>),
    /// MPRIS layer reported a failure; surfaced to the user.
    PlayerError(PlayerError),
    /// Album art was successfully fetched + decoded for the given URL.
    ArtLoaded { url: String, image: Arc<DynamicImage> },
    /// Album-art fetch failed for the given URL.
    ArtFailed { url: String, error: ArtError },
    /// Generic input pump error.
    InputError(String),
}

pub type AppSender = mpsc::Sender<AppEvent>;
pub type AppReceiver = mpsc::Receiver<AppEvent>;

/// Build the application's event channel. The capacity is generous enough
/// that bursts of MPRIS updates can't overrun the loop.
pub fn channel() -> (AppSender, AppReceiver) {
    mpsc::channel(64)
}

/// Spawn a task that forwards terminal events and a steady-rate tick to the
/// shared channel. Exits cleanly when the receiver is dropped.
pub fn spawn_input_pump(tx: AppSender, frame_rate: u16) -> JoinHandle<()> {
    let tick_interval = Duration::from_millis(1_000 / u64::from(frame_rate.clamp(1, 240)));

    tokio::spawn(async move {
        let mut reader = EventStream::new();
        let mut ticker = time::interval(tick_interval);
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            let event = tokio::select! {
                _ = ticker.tick() => AppEvent::Tick,
                maybe = reader.next() => match maybe {
                    Some(Ok(ev)) => AppEvent::Input(ev),
                    Some(Err(err)) => AppEvent::InputError(err.to_string()),
                    None => {
                        debug!("crossterm event stream closed");
                        break;
                    }
                },
            };

            if tx.send(event).await.is_err() {
                break;
            }
        }
    })
}
