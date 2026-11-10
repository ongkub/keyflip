use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Th,
    En,
    Neutral,
}

pub fn is_thai(c: char) -> bool {
    ('\u{e01}'..='\u{e5b}').contains(&c)
}
pub fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic()
}

// Python str.split/strip additionally consider these four control characters whitespace.
pub(crate) fn whitespace(c: char) -> bool {
    c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c)
}

#[derive(Debug, Clone)]
pub struct Keymap {
    to_th: HashMap<String, String>,
    to_en: HashMap<String, String>,
}
impl Keymap {
    pub fn from_pairs(pairs: &[(String, String)]) -> Self {
        Self {
            to_th: pairs.iter().cloned().collect(),
            to_en: pairs.iter().map(|(a, b)| (b.clone(), a.clone())).collect(),
        }
    }
    pub fn mac_default() -> Self {
        let pairs: Vec<(String, String)> = serde_json::from_str(include_str!("../../pairs.json"))
            .expect("valid embedded pairs.json");
        Self::from_pairs(&pairs)
    }
    fn translate(&self, s: &str, table: &HashMap<String, String>) -> String {
        let mut out = String::new();
        for c in s.chars() {
            let mut buffer = [0u8; 4];
            let symbol = c.encode_utf8(&mut buffer);
            out.push_str(table.get(symbol).map(String::as_str).unwrap_or(symbol));
        }
        out
    }
    pub fn to_thai(&self, s: &str) -> String {
        self.translate(s, &self.to_th)
    }
    pub fn to_english(&self, s: &str) -> String {
        self.translate(s, &self.to_en)
    }
    pub fn runs(&self, line: &str) -> Vec<(String, Lang, Option<Lang>)> {
        runs(line)
    }
}

pub fn runs(line: &str) -> Vec<(String, Lang, Option<Lang>)> {
    let (mut result, mut previous) = (Vec::new(), None);
    for word in line.split(whitespace).filter(|s| !s.is_empty()) {
        let mut start = 0;
        let mut last_thai = is_thai(word.chars().next().unwrap());
        for (i, c) in word
            .char_indices()
            .skip(1)
            .chain(std::iter::once((word.len(), '\0')))
        {
            let now = is_thai(c);
            if i == word.len() || now != last_thai {
                let chunk = &word[start..i];
                let lang = if last_thai {
                    Lang::Th
                } else if chunk.chars().any(is_latin) {
                    Lang::En
                } else {
                    Lang::Neutral
                };
                result.push((chunk.to_owned(), lang, previous));
                if lang != Lang::Neutral {
                    previous = Some(lang);
                }
                start = i;
                last_thai = now;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mapping_and_injection() {
        let map = Keymap::mac_default();
        assert_eq!(map.to_thai("vpkdwxdbo-hk;"), "อยากไปกินข้าว");
        assert_eq!(map.to_english(&map.to_thai("meeting")), "meeting");
        assert_eq!(map.to_thai("🙂"), "🙂");
        let custom = Keymap::from_pairs(&[("a".into(), "ก".into()), ("a".into(), "ข".into())]);
        assert_eq!(custom.to_thai("a"), "ข");
        assert_eq!(custom.to_english("กข"), "aa");
    }
    #[test]
    fn splits_like_python() {
        assert_eq!(
            runs("  hiไทย! 123\u{1c}ok ก้ "),
            vec![
                ("hi".into(), Lang::En, None),
                ("ไทย".into(), Lang::Th, Some(Lang::En)),
                ("!".into(), Lang::Neutral, Some(Lang::Th)),
                ("123".into(), Lang::Neutral, Some(Lang::Th)),
                ("ok".into(), Lang::En, Some(Lang::Th)),
                ("ก้".into(), Lang::Th, Some(Lang::En))
            ]
        );
        assert!(runs("").is_empty());
    }
}
