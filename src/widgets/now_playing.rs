//! Track-info pane: title (rendered large), artist(s), album.
//!
//! The title uses [`tui_big_text`] so it stands out at a glance — each glyph
//! is drawn at quadrant resolution (4×4 cells per character), giving roughly
//! "marquee" weight. When the terminal isn't wide or tall enough to fit the
//! big rendering, we fall back to a bold normal-size title so nothing gets
//! clipped.
//!
//! Everything is vertically + horizontally centered inside the pane.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};

use crate::domain::PlayerSnapshot;
use crate::tui::theme::Theme;

/// Cells per glyph at Quadrant resolution. Used to decide whether the title
/// will fit before we commit to rendering it large.
const BIG_GLYPH_W: u16 = 4;
const BIG_GLYPH_H: u16 = 4;

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

    let title_chars = u16::try_from(track.title.chars().count()).unwrap_or(u16::MAX);
    let big_w = title_chars.saturating_mul(BIG_GLYPH_W);

    // Use big text only when we can comfortably fit it. Otherwise the
    // rendering would be clipped mid-glyph, which looks worse than a normal
    // bold title.
    let use_big = inner.width >= big_w.saturating_add(2) && inner.height >= 8;

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
    let content_h: u16 = BIG_GLYPH_H + 1 + 1 + 1; // title + gap + artist + album
    let top_pad = inner.height.saturating_sub(content_h) / 2;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(BIG_GLYPH_H),
            Constraint::Length(1), // gap
            Constraint::Length(1), // artists
            Constraint::Length(1), // album
            Constraint::Min(0),    // bottom pad
        ])
        .split(inner);

    let big = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(theme.primary)
        .lines(vec![Line::from(title.to_string())])
        .centered()
        .build();
    frame.render_widget(big, rows[1]);

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
