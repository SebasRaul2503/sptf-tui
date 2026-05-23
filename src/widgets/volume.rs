//! Volume indicator.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::domain::PlayerSnapshot;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Volume ", theme.primary));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 1 {
        return;
    }

    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(8), Constraint::Min(1), Constraint::Length(6)])
        .split(inner);

    let volume = snapshot.playback.volume;
    let ratio = f64::from(volume) / 100.0;

    let label = Paragraph::new(Span::styled("Vol", theme.muted));
    let gauge = Gauge::default()
        .gauge_style(theme.progress_filled)
        .label("")
        .ratio(ratio.clamp(0.0, 1.0))
        .use_unicode(true);
    let value =
        Paragraph::new(Span::styled(format!("{volume:>3}%"), theme.primary)).right_aligned();

    frame.render_widget(label, row[0]);
    frame.render_widget(gauge, row[1]);
    frame.render_widget(value, row[2]);
}
