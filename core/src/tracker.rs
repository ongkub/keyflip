//! Per-keystroke state machine, matching app/Sources/Core.swift.
//!
//! `grapheme` deletion approximates user-perceived characters by attaching Thai
//! combining marks and Unicode General_Category=M marks to the preceding scalar.
//! A leading sequence of marks counts as one character. This covers Thai and
//! decomposed Latin; it is not full UAX #29 (e.g. emoji ZWJ/flag sequences).
pub use crate::keymap::Lang;
use crate::{
    detect::{Decision, Detector},
    keymap::{is_latin, is_thai, Keymap},
};
use regex::Regex;
use serde::Serialize;
use std::{
    collections::{BTreeSet, HashMap},
    sync::OnceLock,
};

pub trait Decider {
    fn decide(&self, run: &str, prev: Option<Lang>, complete: bool) -> Decision;
}
impl Decider for Detector {
    fn decide(&self, run: &str, prev: Option<Lang>, complete: bool) -> Decision {
        Detector::decide(self, run, prev, complete)
    }
}
impl<T: Decider + ?Sized> Decider for &T {
    fn decide(&self, run: &str, prev: Option<Lang>, complete: bool) -> Decision {
        (**self).decide(run, prev, complete)
    }
}

#[derive(Debug, Clone)]
pub struct Mapper(Keymap);
impl Mapper {
    pub fn new(keymap: Keymap) -> Self {
        Self(keymap)
    }
    pub fn convert(&self, text: &str, to: Lang) -> String {
        match to {
            Lang::Th => self.0.to_thai(text),
            Lang::En => self.0.to_english(text),
            Lang::Neutral => text.to_owned(),
        }
    }
}
impl Default for Mapper {
    fn default() -> Self {
        Self(Keymap::mac_default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(String),
    Space,
    Tab,
    Return,
    Backspace,
    Undo,
    Reset,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Pass,
    Replace {
        delete: usize,
        insert: String,
        switch_to: Lang,
    },
    Revert {
        delete: usize,
        insert: String,
        switch_to: Lang,
    },
}
/// JSON uses an `event` discriminator (`runEnded`, `converted`, `reverted`).
/// Conversion/reversion `kind` is the Swift trigger (`typing`, `space`,
/// `backspace`, `shift`); language values are `th`, `en`, or `neutral`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum TrackerEvent {
    RunEnded {
        lang: String,
    },
    Converted {
        kind: String,
        from: Lang,
        to: Lang,
        score: f64,
        key: Option<usize>,
        prev: Option<Lang>,
        word: String,
        result: String,
    },
    Reverted {
        kind: String,
        from: Lang,
        to: Lang,
        word: String,
        result: String,
    },
}
#[derive(Debug, Clone)]
struct UndoRecord {
    source: Lang,
    target: Lang,
    current_run: String,
    suffix: String,
    active: bool,
    fresh: bool,
}

pub struct RunTracker<D: Decider> {
    decider: D,
    mapper: Mapper,
    delete_mode: String,
    run: String,
    prev: Option<Lang>,
    checked_stopped: bool,
    undo_record: Option<UndoRecord>,
    never: BTreeSet<String>,
    backspace_undos: HashMap<String, usize>,
    events: Vec<TrackerEvent>,
}
fn lang(text: &str) -> Option<Lang> {
    match (text.chars().any(is_thai), text.chars().any(is_latin)) {
        (true, false) => Some(Lang::Th),
        (false, true) => Some(Lang::En),
        _ => None,
    }
}
fn lang_name(lang: Option<Lang>) -> &'static str {
    match lang {
        Some(Lang::Th) => "th",
        Some(Lang::En) => "en",
        _ => "neutral",
    }
}
fn control_characters() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[\p{Cc}\p{Cf}]").expect("valid control regex"))
}
fn combining(c: char) -> bool {
    if matches!(c as u32, 0x0e31 | 0x0e34..=0x0e3a | 0x0e47..=0x0e4e) {
        return true;
    }
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^\p{M}$").expect("valid mark regex"));
    let mut bytes = [0; 4];
    re.is_match(c.encode_utf8(&mut bytes))
}
impl<D: Decider> RunTracker<D> {
    pub fn new(decider: D, mapper: Mapper, never: BTreeSet<String>, delete_mode: &str) -> Self {
        Self {
            decider,
            mapper,
            never,
            delete_mode: delete_mode.into(),
            run: String::new(),
            prev: None,
            checked_stopped: false,
            undo_record: None,
            backspace_undos: HashMap::new(),
            events: Vec::new(),
        }
    }
    pub fn decider(&self) -> &D {
        &self.decider
    }
    pub fn never(&self) -> &BTreeSet<String> {
        &self.never
    }
    /// Replace the persisted never-list; Backspace undo counters remain intact.
    pub fn set_never(&mut self, never: BTreeSet<String>) {
        self.never = never;
    }
    pub fn clear_never(&mut self) {
        self.never.clear();
        self.backspace_undos.clear();
    }
    pub fn take_events(&mut self) -> Vec<TrackerEvent> {
        std::mem::take(&mut self.events)
    }
    fn count(&self, text: &str) -> usize {
        if self.delete_mode != "grapheme" {
            return text.chars().count();
        }
        let mut count = 0;
        for c in text.chars() {
            if count == 0 || !combining(c) {
                count += 1;
            }
        }
        count
    }
    fn emit_ended(&mut self, language: Option<Lang>) {
        self.events.push(TrackerEvent::RunEnded {
            lang: lang_name(language).into(),
        });
    }
    fn finish(&mut self) {
        let language = lang(&self.run);
        self.emit_ended(language);
        if let Some(l) = language {
            self.prev = Some(l);
        }
        self.run.clear();
        self.checked_stopped = false;
    }
    fn should_skip(&self, complete: bool) -> bool {
        self.run.is_empty()
            || self.checked_stopped
            || (self.run.chars().any(is_thai) && self.run.chars().any(is_latin))
            || if complete {
                self.run.chars().count() > 40 || self.never.contains(&self.run)
            } else {
                self.run.chars().count() > 20
                    || self.never.iter().any(|word| word.starts_with(&self.run))
            }
    }
    pub fn handle(&mut self, key: Key) -> Action {
        match key {
            Key::Reset => {
                self.run.clear();
                self.prev = None;
                self.undo_record = None;
                self.checked_stopped = false;
                Action::Pass
            }
            Key::Undo => {
                if let Some(u) = self.undo_record.clone().filter(|u| u.active) {
                    self.revert("shift", u)
                } else {
                    Action::Pass
                }
            }
            Key::Backspace => {
                if let Some(u) = self.undo_record.clone().filter(|u| u.active && u.fresh) {
                    return self.revert("backspace", u);
                }
                if self.run.is_empty() {
                    self.prev = None;
                    self.undo_record = None;
                    self.checked_stopped = false;
                } else {
                    self.run.pop();
                    if let Some(u) = &mut self.undo_record {
                        u.current_run = self.run.clone();
                        u.fresh = false;
                    }
                    self.checked_stopped = false;
                }
                Action::Pass
            }
            Key::Return => {
                self.finish();
                self.undo_record = None;
                self.prev = None;
                Action::Pass
            }
            Key::Space | Key::Tab => {
                let suffix = if key == Key::Space { " " } else { "\t" };
                if self.undo_record.as_ref().is_some_and(|u| u.active) && !self.run.is_empty() {
                    self.undo_record.as_mut().unwrap().suffix.push_str(suffix);
                    let language = lang(&self.run);
                    self.emit_ended(language);
                    if let Some(l) = language {
                        self.prev = Some(l);
                    }
                    self.run.clear();
                    self.checked_stopped = false;
                    return Action::Pass;
                }
                if self.run.is_empty() {
                    if let Some(u) = &mut self.undo_record {
                        u.active = false;
                    }
                }
                if !self.should_skip(true) {
                    let d = self.decider.decide(&self.run, self.prev, true);
                    if d.convert {
                        let source = lang(&self.run).unwrap_or(if d.target == Lang::Th {
                            Lang::En
                        } else {
                            Lang::Th
                        });
                        let n = self.count(&self.run);
                        self.emit_ended(Some(d.target));
                        self.events.push(TrackerEvent::Converted {
                            kind: "space".into(),
                            from: source,
                            to: d.target,
                            score: d.score,
                            key: None,
                            prev: self.prev,
                            word: self.run.clone(),
                            result: d.candidate.clone(),
                        });
                        let inserted = d.candidate.clone() + suffix;
                        self.undo_record = Some(UndoRecord {
                            source,
                            target: d.target,
                            current_run: d.candidate,
                            suffix: suffix.into(),
                            active: true,
                            fresh: true,
                        });
                        self.run.clear();
                        // The run is empty: the next word must be checked from scratch. (This was `true`, so a
                        // word typed right after a space-triggered conversion was never checked.)
                        self.checked_stopped = false;
                        self.prev = Some(d.target);
                        return Action::Replace {
                            delete: n,
                            insert: inserted,
                            switch_to: d.target,
                        };
                    }
                }
                self.finish();
                Action::Pass
            }
            Key::Char(s) => {
                if s.is_empty() || control_characters().is_match(&s) {
                    self.run.clear();
                    self.prev = None;
                    self.undo_record = None;
                    self.checked_stopped = false;
                    return Action::Pass;
                }
                if let Some(u) = &mut self.undo_record {
                    u.fresh = false;
                    if self.run.is_empty() {
                        self.undo_record = None;
                    }
                }
                self.run.push_str(&s);
                if let Some(u) = &mut self.undo_record {
                    if u.active {
                        u.current_run = self.run.clone();
                    }
                }
                if self.should_skip(false) {
                    return Action::Pass;
                }
                let d = self.decider.decide(&self.run, self.prev, false);
                if d.convert {
                    let source = lang(&self.run).unwrap_or(if d.target == Lang::Th {
                        Lang::En
                    } else {
                        Lang::Th
                    });
                    let n = self.count(&self.run);
                    self.events.push(TrackerEvent::Converted {
                        kind: "typing".into(),
                        from: source,
                        to: d.target,
                        score: d.score,
                        key: Some(n),
                        prev: self.prev,
                        word: self.run.clone(),
                        result: d.candidate.clone(),
                    });
                    self.undo_record = Some(UndoRecord {
                        source,
                        target: d.target,
                        current_run: d.candidate.clone(),
                        suffix: String::new(),
                        active: true,
                        fresh: true,
                    });
                    self.run = d.candidate.clone();
                    self.prev = Some(d.target);
                    self.checked_stopped = true;
                    return Action::Replace {
                        delete: n.saturating_sub(1),
                        insert: d.candidate,
                        switch_to: d.target,
                    };
                }
                Action::Pass
            }
        }
    }
    fn revert(&mut self, kind: &str, u: UndoRecord) -> Action {
        let back = self.mapper.convert(&u.current_run, u.source);
        let shown = u.current_run.clone() + &u.suffix;
        if kind == "backspace" {
            *self.backspace_undos.entry(back.clone()).or_default() += 1;
        }
        if kind != "backspace" || self.backspace_undos.get(&back).copied().unwrap_or(0) >= 2 {
            self.never.insert(back.clone());
        }
        self.events.push(TrackerEvent::Reverted {
            kind: kind.into(),
            from: u.target,
            to: u.source,
            word: u.current_run,
            result: back.clone(),
        });
        self.undo_record = None;
        self.prev = Some(u.source);
        if u.suffix.is_empty() {
            self.run = back.clone();
            self.checked_stopped = true;
        } else {
            self.run.clear();
            self.checked_stopped = false;
        }
        Action::Revert {
            delete: self.count(&shown),
            insert: back + &u.suffix,
            switch_to: u.source,
        }
    }
}
