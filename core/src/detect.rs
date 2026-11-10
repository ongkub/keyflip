use crate::{
    keymap::{is_latin, is_thai, whitespace, Keymap, Lang},
    model::Model,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::OnceLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub threshold: f64,
    pub final_threshold: f64,
    pub ctx_delta: f64,
    pub min_len: usize,
    pub min_len_final: Option<usize>,
    pub max_check_len: Option<usize>,
    pub neutral_rule: bool,
    pub orthography_convert: f64,
    pub orthography_keep: f64,
    pub orthography_auto: bool,
    /// Accepted for configuration compatibility. No system dictionary is loaded,
    /// so the word set is empty and this bonus never fires (Python config: 0).
    pub dictionary_bonus: f64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            threshold: 0.65,
            final_threshold: 0.65,
            ctx_delta: 0.15,
            min_len: 2,
            min_len_final: None,
            max_check_len: None,
            neutral_rule: false,
            orthography_convert: 1.8,
            orthography_keep: 1.8,
            orthography_auto: true,
            dictionary_bonus: 0.0,
        }
    }
}
impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }
}

pub fn thai_orthography_ok(s: &str) -> bool {
    // Python's ABOVE_BELOW and TONE are sets of characters from strings WITH
    // spaces, unlike TRIGGERS which calls split(). Preserve this subtlety.
    const TRIGGERS: &str = "ะัาำิีึืฺุู็่้๊๋์ํๅ";
    const ABOVE_BELOW: &str = "ั ิ ี ึ ื ุ ู ฺ ็ ่ ้ ๊ ๋ ์ ํ";
    const TONE: &str = "่ ้ ๊ ๋";
    let mut previous = None;
    let mut prev_tone = false;
    for c in s.chars() {
        if previous.is_none() && TRIGGERS.contains(c) {
            return false;
        }
        if TONE.contains(c) && prev_tone {
            return false;
        }
        if ABOVE_BELOW.contains(c)
            && previous
                .is_some_and(|p| !is_thai(p) && !ABOVE_BELOW.contains(p) && !TONE.contains(p))
        {
            return false;
        }
        previous = Some(c);
        prev_tone = TONE.contains(c);
    }
    true
}

fn decimal_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"^\p{Nd}$").unwrap())
}

fn python_digit(c: char) -> bool {
    // str.isdigit includes Numeric_Type=Digit as well as decimal digits.
    // Rust's is_numeric also accepts fractions/Roman numerals, so is too broad.
    let digit = matches!(c as u32,
        178..=179 | 185 | 4969..=4977 | 6618 | 8304 | 8308..=8313 | 8320..=8329 |
        9312..=9320 | 9332..=9340 | 9352..=9360 | 9450 | 9461..=9469 | 9471 |
        10102..=10110 | 10112..=10120 | 10122..=10130 | 68160..=68163 |
        69216..=69224 | 69714..=69722 | 127232..=127242);
    let mut buffer = [0u8; 4];
    digit || decimal_regex().is_match(c.encode_utf8(&mut buffer))
}

fn number_like(s: &str) -> bool {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| {
            Regex::new(concat!(
                r"\A(?:(?:\$|€|£|฿)[\s\x1c-\x1f]*)?",
                r"(?:\d+|\d{1,3}(?:,\d{3})+(?:\.\d+)?|\d+(?:\.\d+)?|",
                r"\d{1,2}:\d{2}|\d+(?:\.\d+)?%|\d{1,4}(?:[-/]\d{1,4}){1,2}|",
                r"\d{2,4}-\d{2,4}-\d{2,4})",
                r"(?:[\s\x1c-\x1f]*(?:฿|บาท|\$|€|£))?\z"
            ))
            .unwrap()
        })
        .is_match(s)
}

pub fn neutral_candidate_ok(s: &str, keymap: &Keymap) -> bool {
    !s.is_empty()
        && s.chars()
            .any(|c| "[];',./\\=-".contains(c) || python_digit(c))
        && !number_like(s.trim_matches(whitespace))
        && thai_orthography_ok(&keymap.to_thai(s))
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub convert: bool,
    pub score: f64,
    /// Always populated, including keep decisions, using serve.py's conversion
    /// rule: any Thai character => English; otherwise => Thai.
    pub candidate: String,
    pub target: Lang,
}

pub struct Detector {
    pub config: Config,
    th: Model,
    en: Model,
    keymap: Keymap,
}
impl Detector {
    pub fn new(config: Config, th: Model, en: Model, keymap: Keymap) -> Self {
        Self {
            config,
            th,
            en,
            keymap,
        }
    }
    fn model(&self, lang: Lang) -> &Model {
        match lang {
            Lang::Th => &self.th,
            Lang::En => &self.en,
            Lang::Neutral => unreachable!(),
        }
    }
    pub fn decide(&self, run: &str, prev: Option<Lang>, complete: bool) -> Decision {
        let has_thai = run.chars().any(is_thai);
        let target = if has_thai { Lang::En } else { Lang::Th };
        let candidate = if has_thai {
            self.keymap.to_english(run)
        } else {
            self.keymap.to_thai(run)
        };
        let mut decision = Decision {
            convert: false,
            score: 0.0,
            candidate,
            target,
        };
        let source = if has_thai {
            Lang::Th
        } else if run.chars().any(is_latin)
            || (self.config.neutral_rule && neutral_candidate_ok(run, &self.keymap))
        {
            Lang::En
        } else {
            return decision;
        };
        let len = run.chars().count();
        if !complete && self.config.max_check_len.is_some_and(|max| len > max) {
            return decision;
        }
        let target_text = if target == Lang::En {
            decision.candidate.to_lowercase()
        } else {
            decision.candidate.clone()
        };
        let source_text = if source == Lang::En {
            run.to_lowercase()
        } else {
            run.to_owned()
        };
        let mut score = (self.model(target).logprob(&target_text, complete)
            - self.model(source).logprob(&source_text, complete))
            / len.max(1) as f64;
        let mut threshold = if complete {
            self.config.final_threshold
        } else {
            self.config.threshold
        };
        if prev == Some(target) {
            threshold -= self.config.ctx_delta;
        } else if prev == Some(source) {
            threshold += self.config.ctx_delta;
        }
        let thai_side = if source == Lang::Th {
            run
        } else {
            &decision.candidate
        };
        if !thai_orthography_ok(thai_side) {
            if source == Lang::Th {
                score += self.config.orthography_convert;
                if self.config.orthography_auto {
                    decision.convert = true;
                    decision.score = score;
                    return decision;
                }
            } else {
                score -= self.config.orthography_keep;
            }
        }
        // The dictionary word set is intentionally empty; see dictionary_bonus.
        let min_len = if complete {
            self.config.min_len_final.unwrap_or(self.config.min_len)
        } else {
            self.config.min_len
        };
        decision.convert = len >= min_len && score > threshold;
        decision.score = score;
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_orthography() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(config.threshold, 0.65);
        assert_eq!(config.ctx_delta, 0.15);
        assert!(config.orthography_auto);
        assert_eq!(config.min_len_final, None);
        for s in ["", "สวัสดี", "ก้", "aก้", "ก า"] {
            assert!(thai_orthography_ok(s), "{s:?}");
        }
        for s in ["้", "ะก", "ก้่", "a้", "a ", "ก  "] {
            assert!(!thai_orthography_ok(s), "{s:?}");
        }
    }
    #[test]
    fn neutral_numbers_and_symbols() {
        let map = Keymap::mac_default();
        for s in [
            "",
            "123",
            "1,234.50",
            "12:30",
            "50%",
            "2026-11-10",
            "1/2",
            "$ 12",
            "๑๒๓",
            "12 บาท",
            "[  ",
        ] {
            assert!(!neutral_candidate_ok(s, &map), "{s:?}");
        }
        assert!(neutral_candidate_ok("[]", &map));
        assert!(neutral_candidate_ok("8iy[", &map));
        assert!(python_digit('²'));
        assert!(python_digit('๑'));
        assert!(!python_digit('Ⅻ'));
        assert!(!python_digit('½'));
    }
}
