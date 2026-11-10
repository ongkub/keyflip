use keyflip_core::tracker::Key;

#[derive(Clone, Copy, Debug, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

/// Distinguish the layout switch we requested from a user's switch. Resetting
/// tracking must also forget an outstanding request, or a later manual switch
/// could accidentally preserve a word from the old layout.
#[derive(Default)]
pub struct LayoutHistory {
    seen: Option<usize>,
    expected: Option<usize>,
}
impl LayoutHistory {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    pub fn expect(&mut self, layout: usize) {
        self.expected = Some(layout);
    }
    pub fn observe(&mut self, layout: usize) -> bool {
        let changed = self.seen.is_some_and(|old| old != layout);
        let user_changed = changed && self.expected != Some(layout);
        if changed {
            self.expected = None;
        }
        self.seen = Some(layout);
        user_changed
    }
}

pub fn is_shift(vk: u32) -> bool {
    matches!(vk, 0x10 | 0xa0 | 0xa1)
}

pub fn supported_language(langid: u16) -> bool {
    langid == 0x041e || langid & 0x03ff == 0x09
}

pub fn classify(vk: u32, modifiers: Modifiers, text: Option<String>) -> Key {
    if modifiers.ctrl || modifiers.alt || modifiers.win {
        return Key::Reset;
    }
    match vk {
        0x08 => Key::Backspace,
        0x09 => Key::Tab,
        0x0d => Key::Return,
        0x20 => Key::Space,
        0x1b | 0x21..=0x28 | 0x2e => Key::Reset,
        _ => match text {
            Some(s) if !s.is_empty() && !s.chars().any(char::is_control) => Key::Char(s),
            _ => Key::Reset,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_requests_user_changes_and_reset() {
        let mut history = LayoutHistory::default();
        assert!(!history.observe(1));
        history.expect(2);
        assert!(!history.observe(1)); // Posted request has not been processed yet.
        assert!(!history.observe(2)); // Our switch preserves the undo record.
        assert!(history.observe(1)); // A manual switch resets tracking.
        history.expect(2);
        history.clear(); // Navigation, focus changes, disabled state, failure.
        assert!(!history.observe(1));
        assert!(history.observe(2)); // Stale request must not suppress this reset.
    }

    #[test]
    fn tracker_keys_and_navigation() {
        for (vk, key) in [
            (8, Key::Backspace),
            (9, Key::Tab),
            (13, Key::Return),
            (32, Key::Space),
        ] {
            assert_eq!(classify(vk, Modifiers::default(), None), key);
        }
        for vk in [
            0x1b, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2e, 0x70,
        ] {
            assert_eq!(classify(vk, Modifiers::default(), None), Key::Reset);
        }
        assert_eq!(
            classify(65, Modifiers::default(), Some("ก".into())),
            Key::Char("ก".into())
        );
        for text in [None, Some(String::new()), Some("\u{1}".into())] {
            assert_eq!(classify(65, Modifiers::default(), text), Key::Reset);
        }
    }
    #[test]
    fn shortcuts_and_layouts() {
        for m in [
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
            Modifiers {
                alt: true,
                ..Default::default()
            },
            Modifiers {
                win: true,
                ..Default::default()
            },
        ] {
            assert_eq!(classify(32, m, Some(" ".into())), Key::Reset);
        }
        assert!(supported_language(0x041e));
        assert!(supported_language(0x0809));
        assert!(!supported_language(0x0411));
        assert!(is_shift(0xa0) && is_shift(0xa1));
        assert!(!is_shift(65));
    }
}
