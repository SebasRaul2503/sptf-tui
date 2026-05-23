//! Playback controls pane: status icon, time, progress bar.

use std::time::Duration;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge};
use ratatui::Frame;

use crate::domain::{PlaybackStatus, PlayerSnapshot};
use crate::tui::theme::Theme;
use crate::utils::time::format_duration;

const NO_PLAYER: &str = "no player";

pub fn render(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Controls ", theme.primary));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .margin(0)
        .split(inner);

    render_status_line(frame, rows[0], snapshot, theme);
    render_progress(frame, rows[1], snapshot, theme);
}

fn render_status_line(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let status = snapshot.playback.status;
    let (icon, label, style) = match status {
        PlaybackStatus::Playing => ("\u{25b6}", "Playing", theme.accent),
        PlaybackStatus::Paused => ("\u{23f8}", "Paused", theme.status_warn),
        PlaybackStatus::Stopped => ("\u{23f9}", "Stopped", theme.muted),
    };

    let identity = snapshot.identity.as_deref().unwrap_or(NO_PLAYER);
    let length = snapshot.track.as_ref().and_then(|t| t.length).unwrap_or(Duration::ZERO);
    let time_text =
        format!("{} / {}", format_duration(snapshot.playback.position), format_duration(length));

    let left = Line::from(vec![
        Span::styled(icon, style),
        Span::raw("  "),
        Span::styled(label, style),
        Span::styled("  ·  ", theme.muted),
        Span::styled(identity, theme.primary),
    ]);
    let right = Line::from(Span::styled(time_text, theme.primary)).right_aligned();

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    frame.render_widget(ratatui::widgets::Paragraph::new(left), halves[0]);
    frame.render_widget(ratatui::widgets::Paragraph::new(right), halves[1]);
}

fn render_progress(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let length = snapshot.track.as_ref().and_then(|t| t.length);
    let ratio = snapshot.playback.progress_ratio(length).unwrap_or(0.0);

    let gauge = Gauge::default()
        .gauge_style(theme.progress_filled)
        .label(Span::styled(format!("{:.0}%", ratio * 100.0), Style::default()))
        .ratio(ratio)
        .use_unicode(true);
    frame.render_widget(gauge, area);
}
