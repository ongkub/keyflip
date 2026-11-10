/// Portable input plan, including surrogate pairs. Windows glue only translates
/// these records into INPUT structures; no text is sent by a hook callback.
#[derive(Debug, PartialEq, Eq)]
pub enum Stroke {
    Backspace(bool),
    Unicode(u16, bool),
}
pub fn plan(delete: usize, text: &str) -> Vec<Stroke> {
    let mut out = Vec::with_capacity(2 * (delete + text.encode_utf16().count()));
    for _ in 0..delete {
        out.extend([Stroke::Backspace(true), Stroke::Backspace(false)]);
    }
    for unit in text.encode_utf16() {
        out.extend([Stroke::Unicode(unit, true), Stroke::Unicode(unit, false)]);
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deletes_then_unicode_units() {
        assert_eq!(
            plan(1, "ก😀"),
            vec![
                Stroke::Backspace(true),
                Stroke::Backspace(false),
                Stroke::Unicode(0x0e01, true),
                Stroke::Unicode(0x0e01, false),
                Stroke::Unicode(0xd83d, true),
                Stroke::Unicode(0xd83d, false),
                Stroke::Unicode(0xde00, true),
                Stroke::Unicode(0xde00, false)
            ]
        );
        assert!(plan(0, "").is_empty());
    }
}
