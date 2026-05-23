//! Unicode-aware big-text widget for the now-playing title.
//!
//! Every character gets a uniform **4-cell-wide × 4-cell-tall slot**. What
//! goes inside the slot depends on whether we have a bitmap glyph for it:
//!
//! 1. If the character is in `font8x8`'s BASIC / LATIN / HIRAGANA sets we
//!    pack its 8×8 bitmap into 4×4 terminal cells using Unicode quadrant
//!    block characters (`▘▝▖▗▀▄▌▐▙▛▜▟█`…). Result: a chunky "marquee" glyph.
//!
//! 2. Otherwise — kanji, katakana, emoji, anything else the terminal can
//!    paint — we render the literal character centered inside the same 4×4
//!    slot. The terminal supplies its own font for that glyph at its normal
//!    cell size; we just frame it in the same column width so the line
//!    stays aligned with the big-rendered characters around it.
//!
//! Trade-off (intentional): a pure-kanji title looks like a row of
//! normal-size characters each in its own 4×4 frame, not 2-4× larger.
//! Supporting truly *large* kanji would require shipping a CJK bitmap font
//! (see the design discussion in the PR for this change).

use font8x8::{UnicodeFonts, BASIC_FONTS, HIRAGANA_FONTS, LATIN_FONTS};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::widgets::Widget;
use unicode_width::UnicodeWidthChar;

/// Cells that one rendered glyph occupies horizontally.
pub const GLYPH_WIDTH: u16 = 4;
/// Cells that one rendered glyph occupies vertically.
pub const GLYPH_HEIGHT: u16 = 4;

const QUADRANT_SYMBOLS: [char; 16] =
    [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];

/// Return the 8×8 glyph for `c` from any supported font set, or `None`.
///
/// Used internally; also exposed so callers can predict whether a character
/// will get the quadrant rendering or the terminal-native fallback.
pub fn glyph(c: char) -> Option<[u8; 8]> {
    BASIC_FONTS.get(c).or_else(|| LATIN_FONTS.get(c)).or_else(|| HIRAGANA_FONTS.get(c))
}

/// Whether `c` has a glyph available in any supported font set (i.e. will
/// get the big quadrant rendering rather than the fallback).
pub fn is_renderable(c: char) -> bool {
    glyph(c).is_some()
}

/// Whether every character in `text` has a bitmap glyph.
///
/// Renderable strings produce a uniformly-styled "marquee"; non-renderable
/// characters fall through to the per-character native fallback. Used as
/// an *informational* check — not a render gate.
pub fn supports_all(text: &str) -> bool {
    text.chars().all(is_renderable)
}

/// Total width in cells that `text` would occupy when rendered. Each
/// character — supported or not — gets a [`GLYPH_WIDTH`]-cell slot.
pub fn rendered_width(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX).saturating_mul(GLYPH_WIDTH)
}

/// A widget that draws one line of text at "big" resolution.
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

        let painted_w = total_w.min(area.width);
        let start_x = match self.alignment {
            Alignment::Left => area.x,
            Alignment::Center => area.x + area.width.saturating_sub(painted_w) / 2,
            Alignment::Right => area.x + area.width.saturating_sub(painted_w),
        };
        let max_x = area.x.saturating_add(area.width);
        let max_y = area.y.saturating_add(area.height.min(GLYPH_HEIGHT));

        for (idx, &c) in chars.iter().enumerate() {
            let cx =
                start_x.saturating_add(u16::try_from(idx).unwrap_or(0).saturating_mul(GLYPH_WIDTH));
            if cx >= max_x {
                break;
            }
            match glyph(c) {
                Some(g) => paint_quadrant_glyph(buf, g, cx, area.y, max_x, max_y, self.style),
                None => paint_native_char(buf, c, cx, area.y, max_x, max_y, self.style),
            }
        }
    }
}

/// Bitmap-glyph path: pack an 8×8 font glyph into 4×4 quadrant cells.
fn paint_quadrant_glyph(
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

/// Fallback path: blank the 4×4 slot and drop `c` into the middle so the
/// terminal renders it with whatever font it has for that codepoint.
fn paint_native_char(
    buf: &mut Buffer,
    c: char,
    base_x: u16,
    base_y: u16,
    max_x: u16,
    max_y: u16,
    style: Style,
) {
    // Blank every cell of the slot first so a previous frame's content
    // doesn't leak through transparent pixels.
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
            buf[(x, y)].set_char(' ').set_style(style);
        }
    }

    // Center the character horizontally + vertically in the slot. Fullwidth
    // CJK characters take 2 terminal cells, halfwidth take 1 — center each
    // accordingly so kanji/katakana, emoji, and ASCII fallbacks all sit in
    // the middle of their column.
    let char_w = u16::try_from(c.width().unwrap_or(1)).unwrap_or(1).clamp(1, GLYPH_WIDTH);
    let x_off = (GLYPH_WIDTH - char_w) / 2;
    // Middle row of the 4-row band: (4 - 1) / 2 = 1, so the second row.
    // Visually this lands the baseline just below the geometric centre,
    // which is where most CJK / Latin glyphs actually sit.
    let y_off: u16 = 1;

    let x = base_x + x_off;
    let y = base_y + y_off;
    if x >= max_x || y >= max_y {
        return;
    }

    let mut buffer = [0u8; 4];
    let symbol = c.encode_utf8(&mut buffer);
    buf[(x, y)].set_symbol(symbol).set_style(style);
    // Mark the continuation cell of a fullwidth char as empty (ratatui's
    // renderer will skip it because the previous cell has width 2).
    if char_w == 2 && x + 1 < max_x {
        buf[(x + 1, y)].set_symbol("").set_style(style);
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

    fn render(text: &str, width: u16) -> String {
        let backend = TestBackend::new(width, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| f.render_widget(BigTitle::new(text).style(Style::default()), f.area()))
            .unwrap();
        terminal.backend().buffer().content().iter().map(ratatui::buffer::Cell::symbol).collect()
    }

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
        // These don't have 8×8 glyphs in our font chain — they'll go through
        // the native-char fallback path in the renderer.
        for c in ['日', '本', '君', '名', 'ア', 'イ', 'ウ', 'エ', 'オ'] {
            assert!(!is_renderable(c), "{c:?} unexpectedly has a glyph");
        }
    }

    #[test]
    fn supports_all_strings() {
        assert!(supports_all(""));
        assert!(supports_all("Hello"));
        assert!(supports_all("Hola Niño"));
        assert!(supports_all("ありがとう"));
        // Has kanji → reports false (informational only; rendering still works).
        assert!(!supports_all("君の名は"));
        assert!(!supports_all("Track 君"));
    }

    #[test]
    fn rendered_width_counts_chars_not_bytes() {
        assert_eq!(rendered_width(""), 0);
        assert_eq!(rendered_width("Hi"), 8);
        assert_eq!(rendered_width("Niño"), 16);
        assert_eq!(rendered_width("ありがとう"), 20);
        // Even kanji get 4 cells each — uniform columns.
        assert_eq!(rendered_width("君の名は"), 16);
    }

    #[test]
    fn ascii_uses_block_chars() {
        let dump = render("A", 40);
        assert!(
            dump.chars().any(|c| QUADRANT_SYMBOLS.contains(&c) && c != ' '),
            "ASCII should render as block glyphs; got: {dump:?}"
        );
    }

    #[test]
    fn unsupported_chars_appear_as_themselves_inline() {
        // A pure-kanji title should contain the actual kanji character in
        // the buffer (fallback path), not any block glyph.
        let dump = render("君", 40);
        assert!(dump.contains('君'), "kanji should appear as itself; got: {dump:?}");
        // The block glyphs should not be used for an unsupported char (the
        // slot is blanked first, then the char is dropped in the middle).
        assert!(
            !dump.chars().any(|c| QUADRANT_SYMBOLS.contains(&c) && c != ' '),
            "no block glyph expected for pure-kanji input: {dump:?}"
        );
    }

    #[test]
    fn mixed_script_title_renders_both_paths() {
        // Hiragana (supported) + kanji (fallback) + Latin (supported)
        // should all appear: block glyphs *and* the literal kanji in the
        // same line.
        let dump = render("あ君A", 40);
        assert!(dump.contains('君'), "kanji should be present: {dump:?}");
        assert!(
            dump.chars().any(|c| QUADRANT_SYMBOLS.contains(&c) && c != ' '),
            "block glyphs should be present for あ and A: {dump:?}"
        );
    }

    #[test]
    fn katakana_renders_via_fallback() {
        let dump = render("ア", 40);
        assert!(dump.contains('ア'), "katakana should appear via fallback: {dump:?}");
    }

    #[test]
    fn render_is_noop_for_too_small_area() {
        let backend = TestBackend::new(2, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| f.render_widget(BigTitle::new("A君"), f.area())).unwrap();
    }
}
