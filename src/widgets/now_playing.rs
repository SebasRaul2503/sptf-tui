//! Track-info pane: title, artist(s), album.

use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::domain::PlayerSnapshot;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Now Playing ", theme.primary));

    let lines = match snapshot.track.as_ref() {
        Some(track) if !track.title.is_empty() => {
            let mut out = vec![
                Line::from(Span::styled(track.title.clone(), theme.primary)),
                Line::from(Span::styled(track.artists_joined(), theme.accent)),
            ];
            if let Some(album) = track.album.as_ref() {
                out.push(Line::from(Span::styled(album.title.clone(), theme.muted)));
            }
            out
        }
        _ => vec![
            Line::from(Span::styled("No track playing.", theme.status_warn)),
            Line::from(""),
            Line::from(Span::styled("Start something in your player.", theme.muted)),
        ],
    };

    let paragraph =
        Paragraph::new(lines).block(block).alignment(Alignment::Center).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}
