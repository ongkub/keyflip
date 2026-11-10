mod common;
use common::{Models, CONFIG, PAIRS};
use keyflip_core::{Action, Engine, Key, Lang};
#[test]
fn tiny_model_config_pairs_tracker_and_never() {
    let dir = Models::new();
    let mut e = Engine::new(&dir.0, Some(CONFIG), Some(PAIRS)).unwrap();
    assert_eq!(e.detector().config.threshold, -1000.);
    assert_eq!(e.handle(Key::Char("a".into())), Action::Pass);
    assert_eq!(
        e.handle(Key::Char("b".into())),
        Action::Replace {
            delete: 1,
            insert: "กข".into(),
            switch_to: Lang::Th
        }
    );
    assert_eq!(
        e.handle(Key::Undo),
        Action::Revert {
            delete: 2,
            insert: "ab".into(),
            switch_to: Lang::En
        }
    );
    assert_eq!(e.never(), &["ab".into()].into());
    assert_eq!(e.take_events().len(), 2);
    assert!(e.take_events().is_empty());
    e.set_never(["other".into()].into());
    assert_eq!(e.never(), &["other".into()].into());
    e.clear_never();
    assert!(e.never().is_empty());
}
#[test]
fn defaults_initial_never_and_all_detector_config_keys() {
    let dir = Models::new();
    // No config -> the tuned detector/config.json, never the untuned Python constructor defaults.
    let defaults = Engine::new(&dir.0, None, None).unwrap();
    let tuned: serde_json::Value =
        serde_json::from_str(include_str!("../../detector/config.json")).unwrap();
    assert_eq!(
        defaults.detector().config.threshold,
        tuned["threshold"].as_f64().unwrap()
    );
    assert_eq!(
        defaults.detector().config.min_len,
        tuned["min_len"].as_u64().unwrap() as usize
    );
    // A partial config overrides only the keys it names.
    let partial = Engine::new(&dir.0, Some(r#"{"threshold": 9.5}"#), None).unwrap();
    assert_eq!(partial.detector().config.threshold, 9.5);
    assert_eq!(
        partial.detector().config.min_len,
        tuned["min_len"].as_u64().unwrap() as usize
    );
    for key in ["never", "initialNever", "neverList"] {
        let mut e = Engine::new(&dir.0, Some(&format!(r#"{{"{key}":["vpkd"]}}"#)), None).unwrap();
        for c in "vpkd".chars() {
            assert_eq!(e.handle(Key::Char(c.to_string())), Action::Pass);
        }
        assert_eq!(e.handle(Key::Space), Action::Pass);
    }
    let config = include_str!("../../detector/config.json");
    let e = Engine::new(&dir.0, Some(config), None).unwrap();
    let actual = serde_json::to_value(&e.detector().config).unwrap();
    let expected: serde_json::Value = serde_json::from_str(config).unwrap();
    assert_eq!(actual, expected);
    let mut e = Engine::new(
        &dir.0,
        Some(r#"{"deleteMode":"grapheme","threshold":-1000,"min_len":1,"orthography_auto":false}"#),
        None,
    )
    .unwrap();
    assert!(matches!(
        e.handle(Key::Char("ที่ก".into())),
        Action::Replace { delete: 1, .. }
    ));
}
#[test]
fn invalid_model_config_and_pair_errors() {
    let dir = Models::new();
    for json in [
        "[]",
        "null",
        "{",
        r#"{"threshold":"no"}"#,
        r#"{"never":[1]}"#,
        r#"{"deleteMode":"bytes"}"#,
    ] {
        assert!(Engine::new(&dir.0, Some(json), None).is_err(), "{json}");
    }
    for json in [
        "{}",
        "null",
        r#"[["a"]]"#,
        r#"[["a","b","c"]]"#,
        r#"[["a",1]]"#,
        r#"[["a","\u0000"]]"#,
    ] {
        assert!(Engine::new(&dir.0, None, Some(json)).is_err(), "{json}");
    }
    std::fs::remove_file(dir.0.join("en.kfm")).unwrap();
    assert!(Engine::new(&dir.0, None, None)
        .err()
        .unwrap()
        .contains("en.kfm"));
    std::fs::write(dir.0.join("en.kfm"), b"corrupt").unwrap();
    assert!(Engine::new(&dir.0, None, None).is_err());
}
