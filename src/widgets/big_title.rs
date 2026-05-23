//! Unicode-aware big-text widget for the now-playing title.
//!
//! Renders text at "quadrant" resolution: each character from an 8×8 bitmap
//! font is packed into 4×4 terminal cells using the Unicode quadrant block
//! characters (`▘▝▖▗▀▄▌▐▙▛▜▟█`…).
//!
//! Originally we used [`tui-big-text`] for this, but that crate only
//! consults `font8x8::BASIC_FONTS` — anything above U+007F (Spanish ñ,
//! Japanese hiragana, …) returns no glyph and renders as a gap. This module
//! does the same Quadrant rendering but walks a chain of font sets:
//!
//! 1. `BASIC_FONTS`     (U+0000–U+007F) — ASCII
//! 2. `LATIN_FONTS`     (U+00A0–U+00FF) — diacritics: ñ, á, é, í, ó, ú, ü…
//! 3. `HIRAGANA_FONTS`  (U+3040–U+309F) — Japanese hiragana
//!
//! Kanji and katakana have no 8×8 glyphs in `font8x8`, so titles containing
//! them aren't fully renderable — callers use [`supports_all`] to detect
//! this case and fall back to a compact (normal-size) title.

use font8x8::{UnicodeFonts, BASIC_FONTS, HIRAGANA_FONTS, LATIN_FONTS};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::widgets::Widget;

/// Cells that one rendered glyph occupies horizontally.
pub const GLYPH_WIDTH: u16 = 4;
/// Cells that one rendered glyph occupies vertically.
pub const GLYPH_HEIGHT: u16 = 4;

const QUADRANT_SYMBOLS: [char; 16] =
    [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];

/// Return the 8×8 glyph for `c` from any supported font set, or `None`.
pub fn glyph(c: char) -> Option<[u8; 8]> {
    BASIC_FONTS.get(c).or_else(|| LATIN_FONTS.get(c)).or_else(|| HIRAGANA_FONTS.get(c))
}

/// Whether `c` has a glyph available in any supported font set.
pub fn is_renderable(c: char) -> bool {
    glyph(c).is_some()
}

/// Whether every character in `text` is renderable. Empty strings count as
/// renderable (vacuously true).
pub fn supports_all(text: &str) -> bool {
    text.chars().all(is_renderable)
}

/// Total width in cells that `text` would occupy when rendered.
pub fn rendered_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX).saturating_mul(GLYPH_WIDTH)
}

/// A widget that draws one line of text using 4×4 quadrant glyphs.
pub struct BigTitle<'a> {
    text: &'a str,
    style: Style,
    alignment: Alignment,
}

impl<'a> BigTitle<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text, style: Style::default(), alignment: Alignment::Left }
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn centered(mut self) -> Self {
        self.alignment = Alignment::Center;
        self
    }
}

impl Widget for BigTitle<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height < GLYPH_HEIGHT {
            return;
        }

        let chars: Vec<char> = self.text.chars().collect();
        let total_w = u16::try_from(chars.len()).unwrap_or(0).saturating_mul(GLYPH_WIDTH);
        if total_w == 0 {
            return;
        }

        // Center the painted region inside `area` when requested. Truncate
        // alignment offsets so we never paint outside `area`.
        let painted_w = total_w.min(area.width);
        let start_x = match self.alignment {
            Alignment::Left => area.x,
            Alignment::Center => area.x + area.width.saturating_sub(painted_w) / 2,
            Alignment::Right => area.x + area.width.saturating_sub(painted_w),
        };
        let max_x = area.x.saturating_add(area.width);
        let max_y = area.y.saturating_add(area.height.min(GLYPH_HEIGHT));

        for (idx, &c) in chars.iter().enumerate() {
            // Missing glyphs render as blanks (callers should gate on
            // `supports_all` before drawing — but we still write the cells so
            // the area isn't left with stale content from a previous frame).
            let glyph = glyph(c).unwrap_or([0; 8]);
            let cx =
                start_x.saturating_add(u16::try_from(idx).unwrap_or(0).saturating_mul(GLYPH_WIDTH));
            if cx >= max_x {
                break;
            }
            paint_glyph(buf, glyph, cx, area.y, max_x, max_y, self.style);
        }
    }
}

fn paint_glyph(
    buf: &mut Buffer,
    glyph: [u8; 8],
    base_x: u16,
    base_y: u16,
    max_x: u16,
    max_y: u16,
    style: Style,
) {
    for cell_row in 0..GLYPH_HEIGHT {
        let y = base_y + cell_row;
        if y >= max_y {
            break;
        }
        for cell_col in 0..GLYPH_WIDTH {
            let x = base_x + cell_col;
            if x >= max_x {
                break;
            }
            let row_top = (cell_row * 2) as usize;
            let row_bot = row_top + 1;
            let col_left = (cell_col * 2) as u8;
            let col_right = col_left + 1;
            let tl = glyph[row_top] & (1 << col_left);
            let tr = glyph[row_top] & (1 << col_right);
            let bl = glyph[row_bot] & (1 << col_left);
            let br = glyph[row_bot] & (1 << col_right);
            buf[(x, y)].set_char(quadrant(tl, tr, bl, br)).set_style(style);
        }
    }
}

const fn quadrant(tl: u8, tr: u8, bl: u8, br: u8) -> char {
    let tl = if tl > 0 { 1 } else { 0 };
    let tr = if tr > 0 { 1 } else { 0 };
    let bl = if bl > 0 { 1 } else { 0 };
    let br = if br > 0 { 1 } else { 0 };
    QUADRANT_SYMBOLS[tl + (tr << 1) + (bl << 2) + (br << 3)]
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;

    #[test]
    fn ascii_is_renderable() {
        assert!(is_renderable('A'));
        assert!(is_renderable('z'));
        assert!(is_renderable('0'));
        assert!(is_renderable(' '));
        assert!(is_renderable('?'));
    }

    #[test]
    fn spanish_diacritics_are_renderable() {
        for c in ['ñ', 'á', 'é', 'í', 'ó', 'ú', 'ü', 'Ñ', 'Á', '¡', '¿'] {
            assert!(is_renderable(c), "expected {c:?} to be renderable");
        }
    }

    #[test]
    fn hiragana_is_renderable() {
        for c in ['あ', 'り', 'が', 'と', 'う'] {
            assert!(is_renderable(c), "expected {c:?} to be renderable");
        }
    }

    #[test]
    fn kanji_and_katakana_are_not_renderable() {
        // Kanji: no 8x8 glyph exists for any of these.
        for c in ['日', '本', '君', '名'] {
            assert!(!is_renderable(c), "{c:?} unexpectedly has a glyph");
        }
        // Katakana isn't in any of the three font sets we consult.
        for c in ['ア', 'イ', 'ウ', 'エ', 'オ'] {
            assert!(!is_renderable(c), "{c:?} unexpectedly has a glyph");
        }
    }

    #[test]
    fn supports_all_strings() {
        assert!(supports_all(""));
        assert!(supports_all("Hello"));
        assert!(supports_all("Hola Niño"));
        assert!(supports_all("Café del Mar"));
        assert!(supports_all("ありがとう"));
        assert!(!supports_all("君の名は"));
        // Mixed: ASCII + kanji.
        assert!(!supports_all("Track 君"));
    }

    #[test]
    fn rendered_width_counts_chars_not_bytes() {
        assert_eq!(rendered_width(""), 0);
        assert_eq!(rendered_width("Hi"), 8);
        assert_eq!(rendered_width("Niño"), 16); // 4 chars
        assert_eq!(rendered_width("ありがとう"), 20); // 5 chars
    }

    #[test]
    fn render_writes_block_chars_for_ascii() {
        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let widget = BigTitle::new("AB").style(Style::default()).centered();
                f.render_widget(widget, f.area());
            })
            .unwrap();
        // Buffer should contain at least one of our quadrant block chars.
        let dump: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(
            dump.chars().any(|c| QUADRANT_SYMBOLS.contains(&c) && c != ' '),
            "expected at least one block glyph in: {dump:?}"
        );
    }

    #[test]
    fn render_uses_block_chars_for_hiragana() {
        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let widget = BigTitle::new("あ").style(Style::default());
                f.render_widget(widget, f.area());
            })
            .unwrap();
        let dump: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect();
        assert!(
            dump.chars().any(|c| QUADRANT_SYMBOLS.contains(&c) && c != ' '),
            "hiragana should render glyphs; got: {dump:?}"
        );
    }

    #[test]
    fn render_is_noop_for_too_small_area() {
        let backend = TestBackend::new(2, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        // Just must not panic.
        terminal.draw(|f| f.render_widget(BigTitle::new("A"), f.area())).unwrap();
    }
}
