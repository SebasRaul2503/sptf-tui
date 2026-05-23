//! Top-level [`App`] type — owns the event loop.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use ratatui_image::picker::Picker;
use tokio::signal;
use tokio::task::JoinHandle;
use tracing::{debug, info, trace, warn};

use super::events::{channel, spawn_input_pump, AppEvent, AppReceiver, AppSender};
use crate::config::Settings;
use crate::core::error::PlayerError;
use crate::input::{Action, Keymap};
use crate::mpris::{spawn_realtime_watcher, MprisPlayerService};
use crate::rendering::{fetch_image, init_picker, ArtCache};
use crate::services::PlayerService;
use crate::state::AppState;
use crate::tui::{self, theme::Theme};

pub struct App {
    settings: Settings,
    state: AppState,
    keymap: Keymap,
    theme: Theme,
    picker: Picker,
    art_cache: Arc<ArtCache>,
    player: Option<Arc<MprisPlayerService>>,
    tx: AppSender,
    rx: AppReceiver,
    tasks: Vec<JoinHandle<()>>,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let (tx, rx) = channel();
        let keymap = Keymap::from_settings(&settings.keys);
        let theme = Theme::by_name(&settings.ui.theme);
        Self {
            settings,
            state: AppState::default(),
            keymap,
            theme,
            picker: init_picker(),
            art_cache: Arc::new(ArtCache::new()),
            player: None,
            tx,
            rx,
            tasks: Vec::new(),
        }
    }

    /// Run the application until the user requests exit or a signal arrives.
    pub async fn run(&mut self) -> Result<()> {
        self.connect_player().await;

        let mut terminal = tui::setup_terminal()?;
        let result = self.run_loop(&mut terminal).await;

        if let Err(err) = tui::restore_terminal(&mut terminal) {
            warn!(?err, "failed to fully restore terminal");
        }

        self.shutdown();
        result
    }

    async fn connect_player(&mut self) {
        let preferred = self.settings.player.preferred.as_deref();
        match MprisPlayerService::connect(preferred).await {
            Ok(service) => {
                info!(bus = %service.bus_name(), "connected to MPRIS player");
                let snapshot = service.snapshot();
                self.handle_snapshot(snapshot);
                let poll = Duration::from_millis(self.settings.player.position_poll_ms);
                self.tasks.push(spawn_realtime_watcher(service.clone(), self.tx.clone(), poll));
                self.player = Some(service);
            }
            Err(PlayerError::NoPlayerAvailable) => {
                self.state.set_banner("No MPRIS player detected — start one and press r");
            }
            Err(err) => {
                warn!(?err, "failed to connect to MPRIS player");
                self.state.set_banner(format!("MPRIS unavailable: {err}"));
            }
        }
    }

    async fn run_loop(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        self.tasks.push(spawn_input_pump(self.tx.clone(), self.settings.ui.frame_rate));
        debug!(frame_rate = self.settings.ui.frame_rate, "event loop started");

        terminal.draw(|f| tui::view::draw(f, &mut self.state, &self.theme))?;

        loop {
            tokio::select! {
                biased;
                _ = signal::ctrl_c() => {
                    debug!("received Ctrl-C signal");
                    self.state.quit();
                }
                event = self.rx.recv() => match event {
                    Some(event) => self.on_event(event).await,
                    None => {
                        debug!("event channel closed");
                        self.state.quit();
                    }
                },
            }

            if self.state.should_quit {
                break;
            }

            terminal.draw(|f| tui::view::draw(f, &mut self.state, &self.theme))?;
        }

        Ok(())
    }

    async fn on_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => trace!("tick"),
            AppEvent::Input(CrosstermEvent::Key(key)) => {
                if let Some(action) = self.keymap.lookup(key) {
                    self.dispatch(action).await;
                }
            }
            AppEvent::Input(CrosstermEvent::Resize(w, h)) => {
                debug!(width = w, height = h, "resize");
            }
            AppEvent::Input(_) => {}
            AppEvent::PlayerSnapshot(snapshot) => self.handle_snapshot(*snapshot),
            AppEvent::PlayerError(err) => {
                self.state.set_banner(format!("player error: {err}"));
                self.player = None;
                self.state.art = None;
                self.state.art_url = None;
            }
            AppEvent::ArtLoaded { url, image } => {
                // Drop the result if the user has since moved to another track.
                if self.state.current_art_url() == Some(url.as_str()) {
                    self.state.art = Some(self.picker.new_resize_protocol((*image).clone()));
                    self.state.art_url = Some(url.clone());
                    self.art_cache.put(url, image);
                }
            }
            AppEvent::ArtFailed { url, error } => {
                warn!(%url, ?error, "album-art fetch failed");
                if self.state.current_art_url() == Some(url.as_str()) {
                    self.state.art = None;
                }
            }
            AppEvent::InputError(msg) => {
                self.state.set_banner(format!("input error: {msg}"));
            }
        }
    }

    fn handle_snapshot(&mut self, snapshot: crate::domain::PlayerSnapshot) {
        self.state.set_player(snapshot);
        self.state.clear_banner();
        self.refresh_album_art();
    }

    fn refresh_album_art(&mut self) {
        let new_url = self.state.current_art_url().map(str::to_string);

        match new_url {
            None => {
                // Track has no art — clear what we had so the UI stops showing it.
                if self.state.art_url.is_some() {
                    self.state.art = None;
                    self.state.art_url = None;
                }
            }
            Some(url) if Some(url.as_str()) == self.state.art_url.as_deref() => {
                // Same URL we already loaded; nothing to do.
            }
            Some(url) => {
                // Promote a cache hit immediately, or spawn a fetch task.
                if let Some(cached) = self.art_cache.get(&url) {
                    self.state.art = Some(self.picker.new_resize_protocol((*cached).clone()));
                    self.state.art_url = Some(url);
                } else {
                    self.state.art = None;
                    self.state.art_url = Some(url.clone());
                    spawn_art_fetch(self.tx.clone(), url);
                }
            }
        }
    }

    async fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.state.quit(),
            Action::Redraw => {
                self.state.clear_banner();
                if self.player.is_none() {
                    self.connect_player().await;
                }
            }
            other => {
                let Some(player) = self.player.clone() else {
                    self.state.set_banner("no player connected");
                    return;
                };
                let result: Result<(), PlayerError> = match other {
                    Action::TogglePlayPause => player.play_pause().await,
                    Action::Next => player.next().await,
                    Action::Previous => player.previous().await,
                    Action::VolumeUp => {
                        let next = player.snapshot().playback.volume.saturating_add(5).min(100);
                        player.set_volume(next).await
                    }
                    Action::VolumeDown => {
                        let next = player.snapshot().playback.volume.saturating_sub(5);
                        player.set_volume(next).await
                    }
                    Action::SeekForward => player.seek_relative(5).await,
                    Action::SeekBackward => player.seek_relative(-5).await,
                    Action::Quit | Action::Redraw => unreachable!("handled above"),
                };
                if let Err(err) = result {
                    self.state.set_banner(format!("action failed: {err}"));
                }
            }
        }
    }

    fn shutdown(&mut self) {
        for handle in self.tasks.drain(..) {
            handle.abort();
        }
    }
}

fn spawn_art_fetch(tx: AppSender, url: String) {
    tokio::spawn(async move {
        let event = match fetch_image(&url).await {
            Ok(image) => AppEvent::ArtLoaded { url, image },
            Err(error) => AppEvent::ArtFailed { url, error },
        };
        let _ = tx.send(event).await;
    });
}
