//! Track-info pane: title (rendered large), artist(s), album.
//!
//! The title is rendered through [`big_title::BigTitle`](super::big_title),
//! which packs 8×8 bitmap glyphs into 4×4 terminal cells using Unicode
//! quadrant blocks. We use big rendering only when:
//!
//! - the pane has room (≥ `big_w + 2` cells wide, ≥ 8 cells tall), and
//! - every character in the title has a glyph in the supported font sets
//!   (ASCII + Latin-1 diacritics + hiragana). Anything else (kanji,
//!   katakana, emoji, …) falls back to the compact rendering rather than
//!   leaving holes where missing glyphs would be.
//!
//! Everything is vertically + horizontally centered inside the pane.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use super::big_title::{self, BigTitle, GLYPH_HEIGHT};
use crate::domain::PlayerSnapshot;
use crate::tui::theme::Theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, snapshot: &PlayerSnapshot, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Now Playing ", theme.primary));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let track = match snapshot.track.as_ref() {
        Some(t) if !t.title.is_empty() => t,
        _ => {
            render_empty(frame, inner, theme);
            return;
        }
    };

    let artists = track.artists_joined();
    let album = track.album.as_ref().map(|a| a.title.clone()).unwrap_or_default();

    let big_w = big_title::rendered_width(&track.title);
    let fits = inner.width >= big_w.saturating_add(2) && inner.height >= 8;
    let use_big = fits && big_title::supports_all(&track.title);

    if use_big {
        render_big(frame, inner, &track.title, &artists, &album, theme);
    } else {
        render_compact(frame, inner, &track.title, &artists, &album, theme);
    }
}

fn render_big(
    frame: &mut Frame<'_>,
    inner: Rect,
    title: &str,
    artists: &str,
    album: &str,
    theme: &Theme,
) {
    // Vertical layout: top pad / big title (4) / gap (1) / artist (1) /
    // album (1) / bottom pad. We assemble the content stack from the middle
    // so it stays centered when the pane gets taller.
    let content_h: u16 = GLYPH_HEIGHT + 1 + 1 + 1; // title + gap + artist + album
    let top_pad = inner.height.saturating_sub(content_h) / 2;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(GLYPH_HEIGHT),
            Constraint::Length(1), // gap
            Constraint::Length(1), // artists
            Constraint::Length(1), // album
            Constraint::Min(0),    // bottom pad
        ])
        .split(inner);

    frame.render_widget(BigTitle::new(title).style(theme.primary).centered(), rows[1]);

    frame.render_widget(
        Paragraph::new(Span::styled(artists.to_string(), theme.accent))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        rows[3],
    );

    if !album.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(album.to_string(), theme.muted))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            rows[4],
        );
    }
}

fn render_compact(
    frame: &mut Frame<'_>,
    inner: Rect,
    title: &str,
    artists: &str,
    album: &str,
    theme: &Theme,
) {
    let mut lines: Vec<Line<'_>> = vec![
        Line::from(Span::styled(title.to_string(), theme.primary)),
        Line::from(Span::styled(artists.to_string(), theme.accent)),
    ];
    if !album.is_empty() {
        lines.push(Line::from(Span::styled(album.to_string(), theme.muted)));
    }

    let content_h = u16::try_from(lines.len()).unwrap_or(3);
    let top_pad = inner.height.saturating_sub(content_h) / 2;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(content_h),
            Constraint::Min(0),
        ])
        .split(inner);

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, rows[1]);
}

fn render_empty(frame: &mut Frame<'_>, inner: Rect, theme: &Theme) {
    let lines = vec![
        Line::from(Span::styled("No track playing.", theme.status_warn)),
        Line::from(""),
        Line::from(Span::styled("Start something in your player.", theme.muted)),
    ];
    let content_h = u16::try_from(lines.len()).unwrap_or(3);
    let top_pad = inner.height.saturating_sub(content_h) / 2;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(content_h),
            Constraint::Min(0),
        ])
        .split(inner);

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), rows[1]);
}
