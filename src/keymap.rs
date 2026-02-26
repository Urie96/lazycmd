use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mlua::prelude::*;

use crate::Mode;

pub struct Keymap {
    pub mode: Mode,
    pub key_sequence: KeySequence,
    pub callback: LuaFunction,
}

#[derive(Debug, PartialEq)]
pub struct KeySequence(Vec<KeyEvent>);

impl KeySequence {
    pub fn prefix_match(&self, events: &[KeyEvent]) -> bool {
        self.0.len() >= events.len() && &self.0[..events.len()] == events
    }

    pub fn all_match(&self, events: &[KeyEvent]) -> bool {
        self.0 == events
    }
}

impl From<&str> for KeySequence {
    fn from(raw: &str) -> Self {
        let (remaining, modifiers) = extract_modifiers(raw);
        let keyseq = parse_key_code_with_modifiers(remaining, modifiers).unwrap();

        Self(keyseq)
    }
}

// FIXME - seems excessively verbose. Use strum to simplify?
fn parse_key_code_with_modifiers(
    raw: &str,
    mut modifiers: KeyModifiers,
) -> anyhow::Result<Vec<KeyEvent>> {
    let c = match raw.to_lowercase().as_str() {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "backtab" => {
            modifiers.insert(KeyModifiers::SHIFT);
            KeyCode::BackTab
        }
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "hyphen" => KeyCode::Char('-'),
        "minus" => KeyCode::Char('-'),
        "tab" => KeyCode::Tab,
        _ => {
            return Ok(raw
                .chars()
                .map(|c| KeyEvent::new(KeyCode::Char(c), modifiers))
                .collect());
        }
    };
    Ok(vec![KeyEvent::new(c, modifiers)])
}

fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
    let mut modifiers = KeyModifiers::empty();
    let mut current = raw;

    loop {
        match current {
            rest if rest.to_lowercase().starts_with("ctrl-") => {
                modifiers.insert(KeyModifiers::CONTROL);
                current = &rest[5..];
            }
            rest if rest.to_lowercase().starts_with("alt-") => {
                modifiers.insert(KeyModifiers::ALT);
                current = &rest[4..];
            }
            rest if rest.to_lowercase().starts_with("shift-") => {
                modifiers.insert(KeyModifiers::SHIFT);
                current = &rest[6..];
            }
            _ => break, // break out of the loop if no known prefix is detected
        };
    }

    (current, modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_d() {
        let keyseq = KeySequence::from("ctrl-d");
        assert_eq!(keyseq.0.len(), 1);
        assert!(keyseq.0[0].modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(keyseq.0[0].code, KeyCode::Char('d'));
    }

    #[test]
    fn test_ctrl_c() {
        let keyseq = KeySequence::from("ctrl-c");
        assert_eq!(keyseq.0.len(), 1);
        assert!(keyseq.0[0].modifiers.contains(KeyModifiers::CONTROL));
        assert_eq!(keyseq.0[0].code, KeyCode::Char('c'));
    }

    #[test]
    fn test_alt_a() {
        let keyseq = KeySequence::from("alt-a");
        assert_eq!(keyseq.0.len(), 1);
        assert!(keyseq.0[0].modifiers.contains(KeyModifiers::ALT));
        assert_eq!(keyseq.0[0].code, KeyCode::Char('a'));
    }

    #[test]
    fn test_simple_char() {
        let keyseq = KeySequence::from("x");
        assert_eq!(keyseq.0.len(), 1);
        assert_eq!(keyseq.0[0].modifiers, KeyModifiers::empty());
        assert_eq!(keyseq.0[0].code, KeyCode::Char('x'));
    }

    #[test]
    fn test_plain_down() {
        let keyseq = KeySequence::from("down");
        assert_eq!(keyseq.0.len(), 1);
        assert_eq!(keyseq.0[0].code, KeyCode::Down);
    }
}
