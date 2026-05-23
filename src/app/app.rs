//! Top-level [`App`] type — owns the event loop.

use anyhow::Result;
use crossterm::event::Event as CrosstermEvent;
use tokio::signal;
use tracing::{debug, trace};

use super::events::{AppEvent, EventSource};
use crate::config::Settings;
use crate::input::{Action, Keymap};
use crate::state::AppState;
use crate::tui::{self, theme::Theme};

pub struct App {
    settings: Settings,
    state: AppState,
    keymap: Keymap,
    theme: Theme,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            state: AppState::default(),
            keymap: Keymap::new(),
            theme: Theme::default(),
        }
    }

    /// Run the application until the user requests exit or a signal arrives.
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = tui::setup_terminal()?;

        // Ensure the terminal is restored on *any* exit path, including a
        // panic that unwinds through `run`.
        let result = self.run_loop(&mut terminal).await;

        if let Err(err) = tui::restore_terminal(&mut terminal) {
            tracing::warn!(?err, "failed to fully restore terminal");
        }
        result
    }

    async fn run_loop(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        let mut events = EventSource::spawn(self.settings.ui.frame_rate);
        debug!(frame_rate = self.settings.ui.frame_rate, "event loop started");

        // First paint before any events arrive so the user sees something
        // immediately rather than a blank alt-screen.
        terminal.draw(|f| tui::view::draw(f, &self.state, &self.theme))?;

        loop {
            tokio::select! {
                biased;

                // Ctrl-C / SIGINT routes through tokio so it doesn't compete
                // with crossterm's own Ctrl-C handling for stdin.
                _ = signal::ctrl_c() => {
                    debug!("received Ctrl-C signal");
                    self.state.quit();
                }

                event = events.next() => {
                    match event {
                        Some(event) => self.on_event(event),
                        None => {
                            debug!("event source closed");
                            self.state.quit();
                        }
                    }
                }
            }

            if self.state.should_quit {
                break;
            }

            terminal.draw(|f| tui::view::draw(f, &self.state, &self.theme))?;
        }

        Ok(())
    }

    fn on_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => {
                trace!("tick");
            }
            AppEvent::Input(CrosstermEvent::Key(key)) => {
                if let Some(action) = self.keymap.lookup(key) {
                    self.dispatch(action);
                }
            }
            AppEvent::Input(CrosstermEvent::Resize(w, h)) => {
                debug!(width = w, height = h, "resize");
            }
            AppEvent::Input(_) => {
                // Mouse / paste / focus events: ignored for now.
            }
            AppEvent::Error(err) => {
                self.state.set_banner(format!("input error: {err}"));
            }
        }
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => self.state.quit(),
            Action::TogglePlayPause
            | Action::Next
            | Action::Previous
            | Action::VolumeUp
            | Action::VolumeDown
            | Action::SeekForward
            | Action::SeekBackward => {
                // Hook into the player service in iteration 2.
                self.state.set_banner("player not connected yet");
            }
            Action::Redraw => {
                self.state.clear_banner();
            }
        }
    }
}
