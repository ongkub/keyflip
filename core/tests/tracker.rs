//! Port of every check in app/Tests/CoreTests.swift, plus boundary regressions.
use keyflip_core::{
    Action, Config, Decider, Decision, Detector, Key, Keymap, Lang, Mapper, Model, RunTracker,
    TrackerEvent,
};
use std::{cell::RefCell, collections::BTreeSet, path::PathBuf, sync::OnceLock};
type Call = (String, Option<Lang>, bool);
struct Fake {
    calls: RefCell<Vec<Call>>,
    body: Box<dyn Fn(&str, bool) -> Decision>,
}
impl Fake {
    fn new(body: impl Fn(&str, bool) -> Decision + 'static) -> Self {
        Self {
            calls: RefCell::new(vec![]),
            body: Box::new(body),
        }
    }
    fn keep() -> Self {
        Self::new(|_, _| keep())
    }
    fn converting() -> Self {
        Self::new(|s, _| if s == "vpkd" { yes() } else { keep() })
    }
    fn len(&self) -> usize {
        self.calls.borrow().len()
    }
}
impl Decider for Fake {
    fn decide(&self, run: &str, prev: Option<Lang>, complete: bool) -> Decision {
        self.calls.borrow_mut().push((run.into(), prev, complete));
        (self.body)(run, complete)
    }
}
fn keep() -> Decision {
    Decision {
        convert: false,
        score: 0.,
        candidate: String::new(),
        target: Lang::En,
    }
}
fn yes() -> Decision {
    Decision {
        convert: true,
        score: 0.9,
        candidate: "อยาก".into(),
        target: Lang::Th,
    }
}
fn tracker(f: &Fake) -> RunTracker<&Fake> {
    RunTracker::new(f, Mapper::default(), BTreeSet::new(), "scalar")
}
fn type_text<D: Decider>(t: &mut RunTracker<D>, text: &str) -> Vec<Action> {
    text.chars()
        .map(|c| t.handle(Key::Char(c.to_string())))
        .collect()
}
fn replace(delete: usize, insert: &str, switch_to: Lang) -> Action {
    Action::Replace {
        delete,
        insert: insert.into(),
        switch_to,
    }
}
fn revert(delete: usize, insert: &str, switch_to: Lang) -> Action {
    Action::Revert {
        delete,
        insert: insert.into(),
        switch_to,
    }
}
#[test]
fn swift_01_typing_conversion_stops_checks() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    for c in ["v", "p", "k"] {
        assert_eq!(t.handle(Key::Char(c.into())), Action::Pass);
    }
    assert_eq!(
        t.handle(Key::Char("d".into())),
        replace(3, "อยาก", Lang::Th)
    );
    assert_eq!(t.handle(Key::Char("ไ".into())), Action::Pass);
    assert_eq!(t.handle(Key::Space), Action::Pass);
    assert_eq!(f.len(), 4);
}
#[test]
fn swift_02_backspace_undo_and_stopped_checks() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    assert_eq!(t.handle(Key::Backspace), revert(4, "vpkd", Lang::En));
    assert!(!t.never().contains("vpkd"));
    let n = f.len();
    t.handle(Key::Char("v".into()));
    assert_eq!(f.len(), n);
}
#[test]
fn swift_03_complete_conversion_and_shift_undo() {
    let f = Fake::new(|s, complete| {
        if s == "l;ylfu" && complete {
            Decision {
                convert: true,
                score: 1.,
                candidate: "สวัสดี".into(),
                target: Lang::Th,
            }
        } else {
            keep()
        }
    });
    let mut t = tracker(&f);
    type_text(&mut t, "l;ylfu");
    assert_eq!(t.handle(Key::Space), replace(6, "สวัสดี ", Lang::Th));
    assert_eq!(t.handle(Key::Undo), revert(7, "l;ylfu ", Lang::En));
}
#[test]
fn swift_04_never_skips_prefixes_and_equal_complete_word() {
    let f = Fake::keep();
    let mut t = RunTracker::new(&f, Mapper::default(), ["vpkd".into()].into(), "scalar");
    type_text(&mut t, "vpkd");
    t.handle(Key::Space);
    assert_eq!(f.len(), 0);
}
#[test]
fn swift_05_mixed_script_skipped() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "abก");
    assert_eq!(f.len(), 2);
}
#[test]
fn swift_06_reset_starts_fresh() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "vp");
    t.handle(Key::Reset);
    type_text(&mut t, "kd");
    assert_eq!(
        f.calls
            .borrow()
            .iter()
            .map(|c| c.0.as_str())
            .collect::<Vec<_>>(),
        ["v", "vp", "k", "kd"]
    );
    assert_eq!(f.calls.borrow()[2].1, None);
}
#[test]
fn swift_07_prev_language() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "hello");
    t.handle(Key::Space);
    type_text(&mut t, "ท");
    assert_eq!(f.calls.borrow().last().unwrap().1, Some(Lang::En));
}
#[test]
fn swift_08_backspace_removes_scalar() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "vpx");
    t.handle(Key::Backspace);
    type_text(&mut t, "k");
    assert_eq!(f.calls.borrow().last().unwrap().0, "vpk");
}
#[test]
fn swift_09_return_never_checks_and_resets_prev() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "a");
    t.handle(Key::Return);
    assert_eq!(f.len(), 1);
    assert!(f.calls.borrow().iter().all(|c| !c.2));
    type_text(&mut t, "ท");
    assert_eq!(f.calls.borrow().last().unwrap().1, None);
}
#[test]
fn swift_10_typing_cap() {
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, &"a".repeat(21));
    assert_eq!(f.len(), 20);
}
#[test]
fn swift_11_mapper_roundtrip() {
    let m = Mapper::default();
    assert_eq!(m.convert("vpkdwxdbo-hk;", Lang::Th), "อยากไปกินข้าว");
    assert_eq!(m.convert("อยากไปกินข้าว", Lang::En), "vpkdwxdbo-hk;");
}
#[test]
fn swift_12_grapheme_and_scalar_counts() {
    for (mode, delete) in [("grapheme", 1), ("scalar", 3)] {
        let f = Fake::new(|s, _| {
            if s == "ที่ก" {
                Decision {
                    convert: true,
                    score: 1.,
                    candidate: "ข".into(),
                    target: Lang::En,
                }
            } else {
                keep()
            }
        });
        let mut t = RunTracker::new(&f, Mapper::default(), BTreeSet::new(), mode);
        t.handle(Key::Char("ที่".into()));
        assert_eq!(
            t.handle(Key::Char("ก".into())),
            replace(delete, "ข", Lang::En)
        );
    }
}
#[test]
fn swift_13_shift_undo_after_continued_typing() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkdไ");
    assert_eq!(t.handle(Key::Undo), revert(5, "vpkdw", Lang::En));
    assert!(t.never().contains("vpkdw"));
}
#[test]
fn swift_14_backspace_after_continued_typing() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkdไ");
    assert_eq!(t.handle(Key::Backspace), Action::Pass);
    assert_eq!(t.handle(Key::Undo), revert(4, "vpkd", Lang::En));
}
#[test]
fn swift_15_undo_typing_conversion_after_space() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Space);
    assert_eq!(t.handle(Key::Undo), revert(5, "vpkd ", Lang::En));
}
#[test]
fn swift_16_typing_after_space_expires_undo() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Space);
    type_text(&mut t, "ก");
    assert_eq!(t.handle(Key::Undo), Action::Pass);
}
#[test]
fn swift_17_event_words() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkdไ");
    t.handle(Key::Undo);
    let events = t.take_events();
    assert!(
        matches!(&events[0], TrackerEvent::Converted {word, result, ..} if word == "vpkd" && result == "อยาก")
    );
    assert!(
        matches!(&events[1], TrackerEvent::Reverted {word, result, ..} if word == "อยากไ" && result == "vpkdw")
    );
}
#[test]
fn swift_18_next_word_checked_and_run_counted() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Space);
    let n = f.len();
    type_text(&mut t, "l");
    assert_eq!(f.len(), n + 1);
    assert_eq!(
        t.take_events()
            .into_iter()
            .filter_map(|e| if let TrackerEvent::RunEnded { lang } = e {
                Some(lang)
            } else {
                None
            })
            .collect::<Vec<_>>(),
        ["th"]
    );
}
#[test]
fn swift_19_next_word_checked_after_undo_ended_run() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Space);
    t.handle(Key::Undo);
    let n = f.len();
    type_text(&mut t, "x");
    assert_eq!(f.len(), n + 1);
    assert_eq!(f.calls.borrow().last().unwrap().0, "x");
}
#[test]
fn swift_20_second_backspace_undo_blocks() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    for _ in 0..2 {
        t.handle(Key::Reset);
        type_text(&mut t, "vpkd");
        t.handle(Key::Backspace);
    }
    assert!(t.never().contains("vpkd"));
}
#[test]
fn swift_21_shift_undo_blocks_immediately() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Undo);
    assert!(t.never().contains("vpkd"));
}
#[test]
fn swift_22_clear_never_also_clears_backspace_counts() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Undo);
    t.clear_never();
    assert!(t.never().is_empty());
    type_text(&mut t, "x");
    t.handle(Key::Reset);
    type_text(&mut t, "vpkd");
    t.handle(Key::Backspace);
    t.clear_never();
    t.handle(Key::Reset);
    type_text(&mut t, "vpkd");
    t.handle(Key::Backspace);
    assert!(t.never().is_empty());
}
#[test]
fn complete_cap_and_never_prefix_divergence() {
    for (len, complete_calls) in [(20, 1), (21, 1), (40, 1), (41, 0)] {
        let f = Fake::keep();
        let mut t = tracker(&f);
        type_text(&mut t, &"a".repeat(len));
        t.handle(Key::Space);
        assert_eq!(
            f.calls.borrow().iter().filter(|c| c.2).count(),
            complete_calls
        );
    }
    let f = Fake::keep();
    let mut t = tracker(&f);
    t.set_never(["abc".into()].into());
    type_text(&mut t, "ab");
    assert_eq!(f.len(), 0);
    t.handle(Key::Space);
    assert_eq!(f.calls.borrow().as_slice(), &[("ab".into(), None, true)]);
    type_text(&mut t, "abd");
    assert_eq!(f.calls.borrow().last().unwrap().0, "abd");
}
#[test]
fn tab_suffix_fresh_backspace_and_double_delimiter_expiration() {
    let f = Fake::converting();
    let mut t = tracker(&f);
    type_text(&mut t, "vpkd");
    t.handle(Key::Tab);
    assert_eq!(t.handle(Key::Backspace), revert(5, "vpkd\t", Lang::En));
    t.handle(Key::Reset);
    type_text(&mut t, "vpkd");
    t.handle(Key::Tab);
    t.handle(Key::Space);
    assert_eq!(t.handle(Key::Undo), Action::Pass);
    assert_eq!(
        t.take_events()
            .iter()
            .filter(|e| matches!(e, TrackerEvent::RunEnded { .. }))
            .count(),
        3
    );
}
#[test]
fn empty_control_reset_return_and_neutral_prev_events() {
    for control in ["", "\0", "ab\n", "\u{200e}"] {
        let f = Fake::converting();
        let mut t = tracker(&f);
        type_text(&mut t, "vpkd");
        t.handle(Key::Char(control.into()));
        assert_eq!(t.handle(Key::Undo), Action::Pass);
        type_text(&mut t, "a");
        assert_eq!(f.calls.borrow().last().unwrap(), &("a".into(), None, false));
    }
    let f = Fake::keep();
    let mut t = tracker(&f);
    type_text(&mut t, "hi");
    t.handle(Key::Space);
    type_text(&mut t, "123");
    t.handle(Key::Tab);
    t.handle(Key::Space);
    type_text(&mut t, "ก");
    assert_eq!(f.calls.borrow().last().unwrap().1, Some(Lang::En));
    assert_eq!(
        t.take_events(),
        vec![
            TrackerEvent::RunEnded { lang: "en".into() },
            TrackerEvent::RunEnded {
                lang: "neutral".into()
            },
            TrackerEvent::RunEnded {
                lang: "neutral".into()
            }
        ]
    );
    t.handle(Key::Return);
    t.handle(Key::Backspace);
    type_text(&mut t, "a");
    assert_eq!(f.calls.borrow().last().unwrap().1, None);
}
#[test]
fn scalar_backspace_even_in_grapheme_mode_and_latin_marks() {
    let f = Fake::keep();
    let mut t = RunTracker::new(&f, Mapper::default(), BTreeSet::new(), "grapheme");
    t.handle(Key::Char("ที่".into()));
    t.handle(Key::Backspace);
    type_text(&mut t, "ก");
    assert_eq!(f.calls.borrow().last().unwrap().0, "ทีก");
    for s in ["a\u{301}b", "a\u{1ab0}b", "\u{301}\u{302}b"] {
        let f = Fake::new(|_, _| yes());
        let mut t = RunTracker::new(&f, Mapper::default(), BTreeSet::new(), "grapheme");
        assert_eq!(t.handle(Key::Char(s.into())), replace(1, "อยาก", Lang::Th));
    }
}
#[test]
fn full_event_payload_order_and_draining() {
    let f = Fake::new(|s, complete| {
        if s == "vpkd" && complete {
            yes()
        } else {
            keep()
        }
    });
    let mut t = tracker(&f);
    type_text(&mut t, "hello");
    t.handle(Key::Space);
    t.take_events();
    type_text(&mut t, "vpkd");
    t.handle(Key::Tab);
    t.handle(Key::Undo);
    assert_eq!(
        t.take_events(),
        vec![
            TrackerEvent::RunEnded { lang: "th".into() },
            TrackerEvent::Converted {
                kind: "space".into(),
                from: Lang::En,
                to: Lang::Th,
                score: 0.9,
                key: None,
                prev: Some(Lang::En),
                word: "vpkd".into(),
                result: "อยาก".into()
            },
            TrackerEvent::Reverted {
                kind: "shift".into(),
                from: Lang::Th,
                to: Lang::En,
                word: "อยาก".into(),
                result: "vpkd".into()
            }
        ]
    );
    assert!(t.take_events().is_empty());
}
fn real_detector() -> Option<&'static Detector> {
    static D: OnceLock<Option<Detector>> = OnceLock::new();
    D.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        let dir = root.join("models/generic");
        if !dir.join("th.kfm").exists() || !dir.join("en.kfm").exists() {
            eprintln!("skipped tracker real-model test: models/generic/*.kfm absent");
            return None;
        }
        // Use the public engine's default detector settings. Scorer parity tests
        // separately exercise detector/config.json (whose threshold is higher).
        Some(Detector::new(
            Config::default(),
            Model::load(dir.join("th.kfm")).unwrap(),
            Model::load(dir.join("en.kfm")).unwrap(),
            Keymap::mac_default(),
        ))
    })
    .as_ref()
}
// Emulate the shell: a replace suppresses the current physical key, deletes
// preceding displayed scalars, inserts text, then changes the keyboard layout.
fn physical_word<D: Decider>(
    t: &mut RunTracker<D>,
    abc: &str,
    mut layout: Lang,
) -> (String, Vec<(usize, Action)>) {
    let mut shown = String::new();
    let mut replacements = Vec::new();
    let map = Mapper::default();
    for (i, c) in abc.chars().enumerate() {
        let text = if layout == Lang::Th {
            map.convert(&c.to_string(), Lang::Th)
        } else {
            c.to_string()
        };
        let action = t.handle(Key::Char(text.clone()));
        match &action {
            Action::Pass => shown.push_str(&text),
            Action::Replace {
                delete,
                insert,
                switch_to,
            } => {
                for _ in 0..*delete {
                    assert!(shown.pop().is_some());
                }
                shown.push_str(insert);
                layout = *switch_to;
                replacements.push((i + 1, action));
            }
            Action::Revert { .. } => panic!("no undo was sent"),
        }
    }
    (shown, replacements)
}
#[test]
fn real_abc_to_thai_within_first_five_keys() {
    let Some(d) = real_detector() else {
        return;
    };
    let mut t = RunTracker::new(d, Mapper::default(), BTreeSet::new(), "scalar");
    let (shown, replacements) = physical_word(&mut t, "vpkdwxdbo-hk;", Lang::En);
    assert_eq!(shown, "อยากไปกินข้าว");
    assert_eq!(replacements.len(), 1);
    assert!(replacements[0].0 <= 5, "converted at {}", replacements[0].0);
    assert!(matches!(
        replacements[0].1,
        Action::Replace {
            switch_to: Lang::Th,
            ..
        }
    ));
}
#[test]
fn real_thai_typed_meeting_converts_to_english() {
    let Some(d) = real_detector() else {
        return;
    };
    let mut t = RunTracker::new(d, Mapper::default(), BTreeSet::new(), "scalar");
    let (shown, replacements) = physical_word(&mut t, "meeting", Lang::Th);
    assert_eq!(shown, "meeting");
    assert_eq!(replacements.len(), 1);
    assert!(matches!(
        replacements[0].1,
        Action::Replace {
            switch_to: Lang::En,
            ..
        }
    ));
    assert!(t.take_events().iter().any(|e| matches!(e, TrackerEvent::Converted {to: Lang::En, word, result, ..} if Keymap::mac_default().to_english(word) == *result)));
}
#[test]
fn real_correct_thai_and_english_never_convert() {
    let Some(d) = real_detector() else {
        return;
    };
    for text in ["สวัสดีครับ", "hello world"] {
        let mut t = RunTracker::new(d, Mapper::default(), BTreeSet::new(), "scalar");
        for c in text.chars() {
            assert_eq!(
                t.handle(if c == ' ' {
                    Key::Space
                } else {
                    Key::Char(c.to_string())
                }),
                Action::Pass,
                "{text}"
            );
        }
        assert_eq!(t.handle(Key::Space), Action::Pass, "{text}");
        assert!(!t
            .take_events()
            .iter()
            .any(|e| matches!(e, TrackerEvent::Converted { .. })));
    }
}
