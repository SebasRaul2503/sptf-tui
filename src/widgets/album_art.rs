//! Album art widget.
//!
//! Renders the cached [`StatefulProtocol`] when present, or a tasteful
//! placeholder block otherwise (so the layout doesn't reflow as art loads).
//!
//! The image is centered inside the pane: [`StatefulImage`] always paints
//! from the top-left of the rect it's given, and `Resize::Fit` keeps the
//! aspect ratio (so the actual painted area is usually smaller than the
//! pane). We ask the protocol what size it *would* paint at, then offset
//! the rect to put it in the middle.

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
            let resize = Resize::Fit(None);
            let target = p.size_for(&resize, inner);
            let centered = center_rect(target.width, target.height, inner);
            let image = StatefulImage::default().resize(resize);
            frame.render_stateful_widget(image, centered, p);
        }
        None => render_placeholder(frame, inner, has_art_url, theme),
    }
}

fn center_rect(w: u16, h: u16, area: Rect) -> Rect {
    let width = w.min(area.width);
    let height = h.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect { x, y, width, height }
}

fn render_placeholder(frame: &mut Frame<'_>, area: Rect, has_art_url: bool, theme: &Theme) {
    if area.height == 0 || area.width == 0 {
        return;
    }
    let label = if has_art_url { "loading…" } else { "no cover" };
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("\u{266b}", theme.muted)),
        Line::from(""),
        Line::from(Span::styled(label, theme.muted)),
    ];
    // Vertically center the placeholder lines inside the inner rect by
    // computing a centered sub-rect that fits the content.
    let target_h = u16::try_from(lines.len()).unwrap_or(4).min(area.height);
    let centered = center_rect(area.width, target_h, area);
    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, centered);
}
