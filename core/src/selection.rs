//! Conversion for explicitly selected text.
use crate::{
    detect::Detector,
    keymap::{is_latin, is_thai, whitespace, Lang},
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Converted {
    pub text: String,
    pub changed: usize,
    pub lang: Option<Lang>,
}

/// Convert wrong-layout runs in selected text, falling back to an explicit flip
/// when the detector considers every run correct.
pub fn convert_selection(detector: &Detector, text: &str) -> Converted {
    let unchanged = || Converted {
        text: text.to_owned(),
        changed: 0,
        lang: None,
    };
    if text.is_empty() || text.chars().count() > 20_000 {
        return unchanged();
    }

    let mut output = String::with_capacity(text.len());
    let mut previous = None;
    let mut changed = 0;
    let mut last_lang = None;
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let first = rest.chars().next().unwrap();
        if whitespace(first) {
            let end = rest
                .char_indices()
                .find(|(_, c)| !whitespace(*c))
                .map_or(text.len(), |(i, _)| offset + i);
            output.push_str(&text[offset..end]);
            offset = end;
            continue;
        }
        let end = rest
            .char_indices()
            .find(|(_, c)| whitespace(*c))
            .map_or(text.len(), |(i, _)| offset + i);
        let token = &text[offset..end];
        for (run, source, _) in crate::keymap::runs(token) {
            let decision = detector.decide(&run, previous, true);
            if decision.convert {
                output.push_str(&decision.candidate);
                changed += 1;
                last_lang = Some(decision.target);
            } else {
                output.push_str(&run);
            }
            if source != Lang::Neutral {
                // Same as the typing path: after a conversion the context is the language the text is now in.
                previous = Some(if decision.convert {
                    decision.target
                } else {
                    source
                });
            }
        }
        offset = end;
    }
    if changed > 0 {
        return Converted {
            text: output,
            changed,
            lang: last_lang,
        };
    }

    // The user explicitly requested conversion: flip only runs containing letters.
    let mut output = String::with_capacity(text.len());
    let mut changed = 0;
    let mut last_lang = None;
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let first = rest.chars().next().unwrap();
        if whitespace(first) {
            let end = rest
                .char_indices()
                .find(|(_, c)| !whitespace(*c))
                .map_or(text.len(), |(i, _)| offset + i);
            output.push_str(&text[offset..end]);
            offset = end;
            continue;
        }
        let end = rest
            .char_indices()
            .find(|(_, c)| whitespace(*c))
            .map_or(text.len(), |(i, _)| offset + i);
        for (run, _, _) in crate::keymap::runs(&text[offset..end]) {
            if run.chars().any(|c| is_thai(c) || is_latin(c)) {
                let d = detector.decide(&run, None, true);
                output.push_str(&d.candidate);
                changed += 1;
                last_lang = Some(d.target);
            } else {
                output.push_str(&run);
            }
        }
        offset = end;
    }
    Converted {
        text: output,
        changed,
        lang: last_lang,
    }
}
