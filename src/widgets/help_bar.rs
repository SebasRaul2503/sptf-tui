//! Top help bar: app name + a compact keybinding cheatsheet.

use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let title = format!(" sptf-tui v{} ", env!("CARGO_PKG_VERSION"));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(title, theme.accent))
        .title_alignment(Alignment::Left);

    let hints = Line::from(vec![
        keyhint("space", "play/pause", theme),
        sep(theme),
        keyhint("n", "next", theme),
        sep(theme),
        keyhint("b", "prev", theme),
        sep(theme),
        keyhint("\u{2190}\u{2192}", "seek", theme),
        sep(theme),
        keyhint("+/-", "vol", theme),
        sep(theme),
        keyhint("q", "quit", theme),
    ]);

    frame.render_widget(Paragraph::new(hints).block(block).alignment(Alignment::Right), area);
}

fn keyhint(key: &'static str, label: &'static str, theme: &Theme) -> Span<'static> {
    Span::styled(format!("{key} {label}"), theme.muted)
}

fn sep(theme: &Theme) -> Span<'static> {
    Span::styled(" · ", theme.border)
}
