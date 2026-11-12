use keyflip_core::{selection::convert_selection, Engine, Lang};
use std::{
    ffi::{CStr, CString},
    path::PathBuf,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The real models are git-ignored; like tracker.rs and parity.rs, skip behaviour checks when they are absent.
fn real_models() -> Option<PathBuf> {
    let dir = root().join("models/generic");
    (dir.join("th.kfm").exists() && dir.join("en.kfm").exists()).then_some(dir)
}

/// A private copy of the tiny fixture model: one directory per test, so parallel tests never share
/// (or delete) each other's files. Never point cleanup at a real models directory.
fn tiny_models(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keyflip-selection-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for lang in ["th", "en"] {
        std::fs::copy(
            root().join("core/tests/fixtures/tiny.kfm"),
            dir.join(format!("{lang}.kfm")),
        )
        .unwrap();
    }
    dir
}

#[test]
fn requested_selection_examples_and_fallback() {
    let Some(dir) = real_models() else {
        eprintln!("skipped selection real-model test: models/generic/*.kfm absent");
        return;
    };
    let engine = Engine::new(&dir, None, None).unwrap();
    let detector = engine.detector();
    let a = convert_selection(detector, "l;ylfu");
    assert_eq!((a.text.as_str(), a.lang), ("สวัสดี", Some(Lang::Th)));
    let b = convert_selection(detector, "vpkdwxdbo-hk;");
    assert_eq!(b.text, "อยากไปกินข้าว");
    let c = convert_selection(detector, "นัดลูกค้า ทำำะรืเ");
    assert_eq!(
        (c.text.as_str(), c.changed, c.lang),
        ("นัดลูกค้า meeting", 1, Some(Lang::En))
    );
    let d = convert_selection(detector, "l;ylfu\n\tl;ylfu  ");
    assert_eq!(d.text, "สวัสดี\n\tสวัสดี  ");
    let e = convert_selection(detector, "สวัสดี");
    assert_eq!((e.text.as_str(), e.lang), ("l;ylfu", Some(Lang::En)));
    assert_eq!(convert_selection(detector, "l;ylfu 2026").text, "สวัสดี 2026");
    let two = convert_selection(detector, "l;ylfu vpkdwxdbo-hk;");
    assert_eq!((two.text.as_str(), two.changed), ("สวัสดี อยากไปกินข้าว", 2));
    assert_eq!(convert_selection(detector, "2026").text, "2026");
}

#[test]
fn empty_and_oversized_input_are_left_alone() {
    let dir = tiny_models("limits");
    let engine = Engine::new(&dir, None, None).unwrap();
    let empty = convert_selection(engine.detector(), "");
    assert_eq!(
        (empty.text.as_str(), empty.changed, empty.lang),
        ("", 0, None)
    );
    let long = "a".repeat(20_001);
    let big = convert_selection(engine.detector(), &long);
    assert_eq!((big.text == long, big.changed, big.lang), (true, 0, None));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ffi_json_round_trip() {
    let dir = tiny_models("ffi");
    let models = CString::new(dir.to_str().unwrap()).unwrap();
    let mut error = std::ptr::null_mut();
    let handle = unsafe {
        keyflip_core::ffi::kf_engine_new(
            models.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            &mut error,
        )
    };
    assert!(!handle.is_null(), "engine creation failed");
    let input = CString::new("l;ylfu").unwrap();
    let raw = unsafe { keyflip_core::ffi::kf_convert_text(handle, input.as_ptr()) };
    assert!(!raw.is_null());
    let value: serde_json::Value =
        unsafe { serde_json::from_str(CStr::from_ptr(raw).to_str().unwrap()).unwrap() };
    assert!(value["text"].is_string() && value["changed"].is_u64());
    assert!(matches!(value["lang"].as_str(), Some("th") | Some("en")) || value["lang"].is_null());
    assert!(
        unsafe { keyflip_core::ffi::kf_convert_text(std::ptr::null_mut(), input.as_ptr()) }
            .is_null()
    );
    unsafe {
        keyflip_core::ffi::kf_string_free(raw);
        keyflip_core::ffi::kf_engine_free(handle);
    }
    let _ = std::fs::remove_dir_all(dir);
}
