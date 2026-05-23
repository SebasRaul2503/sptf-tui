//! String → [`KeyCombo`] parser used for config-driven keybindings.
//!
//! Syntax: `(modifier-)*key`. Tokens are case-insensitive.
//!
//! - Modifiers: `ctrl` (alias `control`), `alt` (alias `meta`), `shift`.
//! - Named keys: `esc`, `escape`, `tab`, `space`, `enter`, `return`,
//!   `backspace`, `left`, `right`, `up`, `down`, `home`, `end`,
//!   `pageup`, `pagedown`, `delete`, `insert`.
//! - Function keys: `f1`–`f12`.
//! - Single-char keys: the character itself (e.g. `q`, `+`, `<`).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

/// Hashable, modifier-aware key identifier.
///
/// Crossterm's [`KeyEvent`] carries `kind` and `state` fields that vary by
/// terminal and would make equality fragile. We strip those to a stable
/// (code, modifiers) pair for lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombo {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

impl From<KeyEvent> for KeyCombo {
    fn from(ev: KeyEvent) -> Self {
        Self { code: ev.code, modifiers: ev.modifiers }
    }
}

impl From<KeyCombo> for KeyEvent {
    fn from(combo: KeyCombo) -> Self {
        Self {
            code: combo.code,
            modifiers: combo.modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}

/// Parse a string like `ctrl-c`, `shift-tab`, `f1`, `q`, `+` into a key combo.
pub fn parse(raw: &str) -> Option<KeyCombo> {
    let s = raw.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split('-').collect();
    let (mod_parts, key_part) = parts.split_at(parts.len() - 1);

    let mut modifiers = KeyModifiers::empty();
    for m in mod_parts {
        match *m {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" | "meta" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            _ => return None,
        }
    }

    let code = parse_keycode(key_part[0])?;
    Some(KeyCombo { code, modifiers })
}

fn parse_keycode(token: &str) -> Option<KeyCode> {
    Some(match token {
        "esc" | "escape" => KeyCode::Esc,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Char(' '),
        "enter" | "return" => KeyCode::Enter,
        "backspace" => KeyCode::Backspace,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "page_up" => KeyCode::PageUp,
        "pagedown" | "page_down" => KeyCode::PageDown,
        "delete" | "del" => KeyCode::Delete,
        "insert" | "ins" => KeyCode::Insert,
        t if t.starts_with('f') && t.len() <= 3 => {
            let n: u8 = t[1..].parse().ok()?;
            if n == 0 || n > 24 {
                return None;
            }
            KeyCode::F(n)
        }
        t if t.chars().count() == 1 => KeyCode::Char(t.chars().next().unwrap()),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_eq(raw: &str, expected: KeyCombo) {
        assert_eq!(parse(raw), Some(expected), "parsing {raw:?}");
    }

    #[test]
    fn parses_single_chars() {
        parse_eq("q", KeyCombo::new(KeyCode::Char('q'), KeyModifiers::empty()));
        parse_eq("+", KeyCombo::new(KeyCode::Char('+'), KeyModifiers::empty()));
        parse_eq("<", KeyCombo::new(KeyCode::Char('<'), KeyModifiers::empty()));
    }

    #[test]
    fn parses_named_keys() {
        parse_eq("esc", KeyCombo::new(KeyCode::Esc, KeyModifiers::empty()));
        parse_eq("escape", KeyCombo::new(KeyCode::Esc, KeyModifiers::empty()));
        parse_eq("space", KeyCombo::new(KeyCode::Char(' '), KeyModifiers::empty()));
        parse_eq("enter", KeyCombo::new(KeyCode::Enter, KeyModifiers::empty()));
        parse_eq("right", KeyCombo::new(KeyCode::Right, KeyModifiers::empty()));
    }

    #[test]
    fn parses_function_keys() {
        parse_eq("f1", KeyCombo::new(KeyCode::F(1), KeyModifiers::empty()));
        parse_eq("f12", KeyCombo::new(KeyCode::F(12), KeyModifiers::empty()));
        assert!(parse("f99").is_none());
        assert!(parse("f0").is_none());
    }

    #[test]
    fn parses_modifiers() {
        parse_eq("ctrl-c", KeyCombo::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        parse_eq(
            "ctrl-shift-x",
            KeyCombo::new(KeyCode::Char('x'), KeyModifiers::CONTROL | KeyModifiers::SHIFT),
        );
        parse_eq("alt-tab", KeyCombo::new(KeyCode::Tab, KeyModifiers::ALT));
    }

    #[test]
    fn is_case_insensitive() {
        parse_eq("CTRL-C", KeyCombo::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        parse_eq("Esc", KeyCombo::new(KeyCode::Esc, KeyModifiers::empty()));
    }

    #[test]
    fn rejects_unknown_modifier() {
        assert!(parse("hyper-c").is_none());
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert!(parse("").is_none());
        assert!(parse("   ").is_none());
        assert!(parse("nope").is_none());
        assert!(parse("ctrl-").is_none()); // empty key part after split
    }
}
