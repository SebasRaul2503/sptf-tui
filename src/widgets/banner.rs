//! Footer status banner.

use ratatui::layout::{Alignment, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let text = state.status_banner.clone().unwrap_or_else(|| describe_default(state));

    let style = if state.status_banner.is_some() { theme.status_warn } else { theme.muted };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border);
    let paragraph =
        Paragraph::new(Span::styled(text, style)).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn describe_default(state: &AppState) -> String {
    if state.player.is_connected() {
        let id = state.player.identity.as_deref().unwrap_or("MPRIS player");
        format!("connected to {id}")
    } else {
        "no player connected — start Spotify (or any MPRIS player)".to_string()
    }
}
