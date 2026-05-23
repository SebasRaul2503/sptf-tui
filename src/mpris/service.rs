//! MPRIS-backed implementation of [`PlayerService`].
//!
//! The service owns the DBus connection and the per-player proxies. It keeps
//! the most recently observed [`PlayerSnapshot`] behind an `RwLock` so the
//! UI's `snapshot()` call is always cheap and infallible. A separate watcher
//! task ([`spawn_watcher`]) periodically refreshes the cache and pushes each
//! new snapshot onto the app's event channel.
//!
//! Iteration 4 will replace the polling watcher with DBus-signal-driven
//! updates; the service API stays the same.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, warn};
use zbus::Connection;

use super::discovery::{list_mpris_players, select_player};
use super::parser::{parse_metadata, parse_status, parse_volume};
use super::proxy::{MediaPlayer2Proxy, PlayerProxy};
use crate::app::{AppEvent, AppSender};
use crate::core::error::PlayerError;
use crate::domain::{PlaybackState, PlayerSnapshot};
use crate::services::PlayerService;

pub struct MprisPlayerService {
    bus_name: String,
    player: PlayerProxy<'static>,
    media: MediaPlayer2Proxy<'static>,
    snapshot: RwLock<PlayerSnapshot>,
}

impl MprisPlayerService {
    /// Connect to the session bus, find an MPRIS player matching `preferred`,
    /// and prime the snapshot cache with one refresh.
    pub async fn connect(preferred: Option<&str>) -> Result<Arc<Self>, PlayerError> {
        let conn = Connection::session().await.map_err(zbus_err)?;
        let names = list_mpris_players(&conn).await?;
        let chosen =
            select_player(&names, preferred).ok_or(PlayerError::NoPlayerAvailable)?.to_string();

        debug!(bus = %chosen, "connecting to MPRIS player");

        let player = PlayerProxy::builder(&conn)
            .destination(chosen.clone())
            .map_err(zbus_err)?
            .build()
            .await
            .map_err(zbus_err)?;
        let media = MediaPlayer2Proxy::builder(&conn)
            .destination(chosen.clone())
            .map_err(zbus_err)?
            .build()
            .await
            .map_err(zbus_err)?;

        let service = Arc::new(Self {
            bus_name: chosen,
            player,
            media,
            snapshot: RwLock::new(PlayerSnapshot::default()),
        });
        service.refresh().await?;
        Ok(service)
    }

    /// Read every observable property and update the cached snapshot.
    pub async fn refresh(&self) -> Result<PlayerSnapshot, PlayerError> {
        let metadata = self.player.metadata().await.map_err(zbus_err)?;
        let status_raw = self.player.playback_status().await.map_err(zbus_err)?;

        // These properties are *allowed* to fail (the spec marks them optional
        // and some players, including Spotify, sporadically return errors).
        // Treat any read failure as "value not available".
        let position = self.player.position().await.unwrap_or(0).max(0);
        let volume = self.player.volume().await.unwrap_or(1.0);
        let identity = self.media.identity().await.ok();

        let track = parse_metadata(&metadata);
        let snapshot = PlayerSnapshot {
            bus_name: Some(self.bus_name.clone()),
            identity,
            track: Some(track),
            playback: PlaybackState {
                status: parse_status(&status_raw),
                position: Duration::from_micros(u64::try_from(position).unwrap_or(0)),
                volume: parse_volume(volume),
            },
        };

        if let Ok(mut guard) = self.snapshot.write() {
            *guard = snapshot.clone();
        }
        Ok(snapshot)
    }

    /// Bus name the service is currently bound to.
    pub fn bus_name(&self) -> &str {
        &self.bus_name
    }
}

#[async_trait]
impl PlayerService for MprisPlayerService {
    fn snapshot(&self) -> PlayerSnapshot {
        self.snapshot.read().map(|g| g.clone()).unwrap_or_default()
    }

    async fn play_pause(&self) -> Result<(), PlayerError> {
        self.player.play_pause().await.map_err(zbus_err)
    }

    async fn next(&self) -> Result<(), PlayerError> {
        self.player.next().await.map_err(zbus_err)
    }

    async fn previous(&self) -> Result<(), PlayerError> {
        self.player.previous().await.map_err(zbus_err)
    }

    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let v = f64::from(volume.min(100)) / 100.0;
        self.player.set_volume(v).await.map_err(zbus_err)
    }

    async fn seek_relative(&self, delta_secs: i64) -> Result<(), PlayerError> {
        let offset_usec = delta_secs.saturating_mul(1_000_000);
        self.player.seek(offset_usec).await.map_err(zbus_err)
    }
}

/// Spawn a task that periodically refreshes `service` and posts the result on
/// `tx`. Stops on receiver shutdown or on a non-recoverable DBus error.
pub fn spawn_watcher(
    service: Arc<MprisPlayerService>,
    tx: AppSender,
    poll_interval: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = time::interval(poll_interval.max(Duration::from_millis(100)));
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            ticker.tick().await;
            match service.refresh().await {
                Ok(snapshot) => {
                    if tx.send(AppEvent::PlayerSnapshot(Box::new(snapshot))).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    warn!(?err, "MPRIS refresh failed");
                    let _ = tx.send(AppEvent::PlayerError(err)).await;
                    break;
                }
            }
        }
        debug!("MPRIS watcher exiting");
    })
}

fn zbus_err<E: std::fmt::Display>(err: E) -> PlayerError {
    PlayerError::Dbus(err.to_string())
}
