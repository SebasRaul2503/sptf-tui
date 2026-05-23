//! MPRIS-backed implementation of [`PlayerService`].
//!
//! The service owns the DBus connection and the per-player proxies. It keeps
//! the most recently observed [`PlayerSnapshot`] behind an `RwLock` so the
//! UI's `snapshot()` call is always cheap and infallible.
//!
//! [`spawn_realtime_watcher`] subscribes to:
//!
//! - `org.freedesktop.DBus.Properties.PropertiesChanged` on the player path,
//!   to receive metadata / status / volume changes the *moment* they happen.
//! - `org.freedesktop.DBus.NameOwnerChanged` filtered to the chosen bus name,
//!   so we surface a clean disconnect instead of stale data.
//! - A low-rate position poll (configurable, default 500 ms) because MPRIS
//!   does not push position updates between `Seeked` signals.
//!
//! Errors during the watcher's lifetime are pushed on the app event channel
//! and the watcher exits; iteration-8 polish handles automatic reconnect.

use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, trace, warn};
use zbus::fdo::{DBusProxy, PropertiesProxy};
use zbus::names::InterfaceName;
use zbus::Connection;

use super::discovery::{list_mpris_players, select_player};
use super::parser::{parse_metadata, parse_status, parse_volume};
use super::proxy::{MediaPlayer2Proxy, PlayerProxy};
use crate::app::{AppEvent, AppSender};
use crate::core::error::PlayerError;
use crate::domain::{PlaybackState, PlayerSnapshot};
use crate::services::PlayerService;

const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER_IFACE: &str = "org.mpris.MediaPlayer2.Player";

pub struct MprisPlayerService {
    bus_name: String,
    conn: Connection,
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
            conn,
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

        // The next three are optional in the MPRIS spec and several players
        // (notably Spotify) sporadically error on them. Read-failure is mapped
        // to a sane default rather than aborting the whole refresh.
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

        self.store_snapshot(&snapshot);
        Ok(snapshot)
    }

    /// Cheap refresh that only re-reads `Position`. Used by the position
    /// poller — saves three DBus round-trips per tick.
    pub async fn refresh_position(&self) -> Result<PlayerSnapshot, PlayerError> {
        let position = self.player.position().await.map_err(zbus_err)?.max(0);
        let new_position = Duration::from_micros(u64::try_from(position).unwrap_or(0));

        let mut snapshot = self.snapshot.read().map(|g| g.clone()).unwrap_or_default();
        snapshot.playback.position = new_position;
        if let Ok(mut guard) = self.snapshot.write() {
            guard.playback.position = new_position;
        }
        Ok(snapshot)
    }

    pub fn bus_name(&self) -> &str {
        &self.bus_name
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    fn store_snapshot(&self, snapshot: &PlayerSnapshot) {
        if let Ok(mut guard) = self.snapshot.write() {
            *guard = snapshot.clone();
        }
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
        let clamped = volume.min(100);
        let v = f64::from(clamped) / 100.0;
        self.player.set_volume(v).await.map_err(zbus_err)?;
        // Some players (Spotify in particular) do not emit PropertiesChanged
        // for Volume changes — so the next "calculate target from cached
        // value" call would keep computing from the stale value, making
        // repeated +/- presses no-ops. Update the cache optimistically; the
        // signal-driven refresh will overwrite when/if the player publishes.
        if let Ok(mut g) = self.snapshot.write() {
            g.playback.volume = clamped;
        }
        Ok(())
    }

    async fn seek_relative(&self, delta_secs: i64) -> Result<(), PlayerError> {
        let offset_usec = delta_secs.saturating_mul(1_000_000);
        self.player.seek(offset_usec).await.map_err(zbus_err)
    }
}

/// Signal-driven watcher: PropertiesChanged + NameOwnerChanged + a
/// low-rate position poll. Replaces the iteration-2 polling loop.
pub fn spawn_realtime_watcher(
    service: Arc<MprisPlayerService>,
    tx: AppSender,
    position_poll: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let conn = service.connection().clone();

        let props_proxy = match build_properties_proxy(&conn, service.bus_name()).await {
            Ok(p) => p,
            Err(err) => {
                let _ = tx.send(AppEvent::PlayerError(err)).await;
                return;
            }
        };

        let mut props_stream = match props_proxy.receive_properties_changed().await {
            Ok(s) => s,
            Err(err) => {
                let _ = tx.send(AppEvent::PlayerError(zbus_err(err))).await;
                return;
            }
        };

        let dbus = match DBusProxy::new(&conn).await {
            Ok(d) => d,
            Err(err) => {
                let _ = tx.send(AppEvent::PlayerError(zbus_err(err))).await;
                return;
            }
        };
        let mut owner_stream = match dbus.receive_name_owner_changed().await {
            Ok(s) => s,
            Err(err) => {
                let _ = tx.send(AppEvent::PlayerError(zbus_err(err))).await;
                return;
            }
        };

        let player_iface = InterfaceName::try_from(PLAYER_IFACE).expect("valid interface name");
        let mut ticker = time::interval(position_poll.max(Duration::from_millis(100)));
        ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        debug!(bus = %service.bus_name(), "MPRIS realtime watcher started");

        loop {
            tokio::select! {
                Some(signal) = props_stream.next() => {
                    let Ok(args) = signal.args() else { continue };
                    if args.interface_name != player_iface {
                        continue;
                    }
                    trace!("PropertiesChanged on Player interface");
                    match service.refresh().await {
                        Ok(snapshot) => {
                            if tx.send(AppEvent::PlayerSnapshot(Box::new(snapshot))).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            warn!(?err, "refresh after PropertiesChanged failed");
                            let _ = tx.send(AppEvent::PlayerError(err)).await;
                            break;
                        }
                    }
                }
                Some(signal) = owner_stream.next() => {
                    let Ok(args) = signal.args() else { continue };
                    if args.name.as_str() == service.bus_name() && args.new_owner.is_none() {
                        debug!(bus = %service.bus_name(), "player lost name ownership");
                        let _ = tx.send(AppEvent::PlayerError(PlayerError::PlayerDisconnected)).await;
                        break;
                    }
                }
                _ = ticker.tick() => {
                    match service.refresh_position().await {
                        Ok(snapshot) => {
                            if tx.send(AppEvent::PlayerSnapshot(Box::new(snapshot))).await.is_err() {
                                break;
                            }
                        }
                        Err(err) => {
                            warn!(?err, "position refresh failed");
                            let _ = tx.send(AppEvent::PlayerError(err)).await;
                            break;
                        }
                    }
                }
                else => break,
            }
        }
        debug!("MPRIS realtime watcher exiting");
    })
}

async fn build_properties_proxy(
    conn: &Connection,
    bus_name: &str,
) -> Result<PropertiesProxy<'static>, PlayerError> {
    PropertiesProxy::builder(conn)
        .destination(bus_name.to_string())
        .map_err(zbus_err)?
        .path(MPRIS_PATH)
        .map_err(zbus_err)?
        .build()
        .await
        .map_err(zbus_err)
}

fn zbus_err<E: std::fmt::Display>(err: E) -> PlayerError {
    PlayerError::Dbus(err.to_string())
}
