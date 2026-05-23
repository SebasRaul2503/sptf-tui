//! Album art widget.
//!
//! Renders the cached [`StatefulProtocol`] when present, or a tasteful
//! placeholder block otherwise (so the layout doesn't reflow as art loads).

use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::{Resize, StatefulImage};

use crate::tui::theme::Theme;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    protocol: Option<&mut StatefulProtocol>,
    has_art_url: bool,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Cover ", theme.primary));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match protocol {
        Some(p) => {
            let image = StatefulImage::default().resize(Resize::Fit(None));
            frame.render_stateful_widget(image, inner, p);
        }
        None => render_placeholder(frame, inner, has_art_url, theme),
    }
}

fn render_placeholder(frame: &mut Frame<'_>, area: Rect, has_art_url: bool, theme: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let label = if has_art_url { "loading…" } else { "no cover" };
    let lines = vec![
        Line::from(Span::styled("\u{266b}", theme.muted)), // musical note
        Line::from(""),
        Line::from(Span::styled(label, theme.muted)),
    ];
    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
