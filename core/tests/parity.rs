use keyflip_core::{Config, Detector, Keymap, Lang, Model};
use serde::Deserialize;
use std::{fs, hint::black_box, path::PathBuf, sync::OnceLock, time::Instant};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}
fn generic_detector() -> Option<&'static Detector> {
    static DETECTOR: OnceLock<Option<Detector>> = OnceLock::new();
    DETECTOR.get_or_init(|| {
        let models = root().join("models/generic");
        if !models.join("th.kfm").exists() || !models.join("en.kfm").exists() {
            eprintln!("skipped: run detector/export_model.py --in detector/models --out models/generic");
            return None;
        }
        Some(Detector::new(Config::load(root().join("detector/config.json")).unwrap(),
            Model::load(models.join("th.kfm")).unwrap(), Model::load(models.join("en.kfm")).unwrap(), Keymap::mac_default()))
    }).as_ref()
}
fn jsonl<T: serde::de::DeserializeOwned>(name: &str) -> Vec<T> {
    let path = root().join("core/tests/vectors").join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| {
            panic!(
                "{}: {e}; run python3 detector/export_vectors.py",
                path.display()
            )
        })
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
#[derive(Debug, Deserialize)]
struct DecideVector {
    run: String,
    prev: Option<Lang>,
    complete: bool,
    convert: bool,
    score: f64,
    candidate: String,
    target: Lang,
}
fn assert_decision(detector: &Detector, row: &DecideVector) -> f64 {
    let actual = detector.decide(&row.run, row.prev, row.complete);
    assert_eq!(actual.convert, row.convert, "{row:?}; {actual:?}");
    assert_eq!(actual.candidate, row.candidate, "{row:?}");
    assert_eq!(actual.target, row.target, "{row:?}");
    let difference = (actual.score - row.score).abs();
    assert!(
        difference <= 1e-3,
        "difference {difference}: {row:?}; {actual:?}"
    );
    difference
}
#[test]
fn decide_parity() {
    let Some(detector) = generic_detector() else {
        return;
    };
    let rows: Vec<DecideVector> = jsonl("decide.jsonl");
    assert!(!rows.is_empty());
    let maximum = rows
        .iter()
        .map(|row| assert_decision(detector, row))
        .fold(0.0, f64::max);
    println!(
        "decide parity: {} vectors, max score difference {maximum:.9}",
        rows.len()
    );
}
#[derive(Deserialize)]
struct LogVector {
    lang: Lang,
    run: String,
    complete: bool,
    score: f64,
}
#[test]
fn logprob_parity() {
    if generic_detector().is_none() {
        return;
    }
    let th = Model::load(root().join("models/generic/th.kfm")).unwrap();
    let en = Model::load(root().join("models/generic/en.kfm")).unwrap();
    let rows: Vec<LogVector> = jsonl("logprob.jsonl");
    assert!(rows.len() >= 8000);
    let mut maximum = 0.0_f64;
    for row in &rows {
        let model = if row.lang == Lang::Th { &th } else { &en };
        let value = model.logprob(&row.run, row.complete);
        let diff = (value - row.score).abs();
        assert!(
            diff <= 1e-4,
            "difference {diff}, {:?}, {:?}, complete={}: Rust {value} Python {}",
            row.lang,
            row.run,
            row.complete,
            row.score
        );
        maximum = maximum.max(diff);
    }
    println!(
        "logprob parity: {} vectors, max difference {maximum:.9}",
        rows.len()
    );
}
#[test]
fn decide_speed() {
    let Some(detector) = generic_detector() else {
        return;
    };
    let rows: Vec<DecideVector> = jsonl("decide.jsonl");
    // Warm one-time regex initialization before timing the scoring workload.
    for row in &rows {
        black_box(detector.decide(&row.run, row.prev, row.complete));
    }
    let start = Instant::now();
    for row in &rows {
        black_box(detector.decide(black_box(&row.run), row.prev, row.complete));
    }
    let mean_ms = start.elapsed().as_secs_f64() * 1000.0 / rows.len() as f64;
    println!(
        "decide mean: {mean_ms:.6} ms ({} vectors, release={})",
        rows.len(),
        !cfg!(debug_assertions)
    );
    if !cfg!(debug_assertions) {
        assert!(mean_ms < 0.2, "decide mean {mean_ms} ms >= 0.2 ms");
    }
}
#[derive(Deserialize)]
struct TinyLog {
    run: String,
    complete: bool,
    score: f64,
}
#[test]
fn synthetic_model_without_generic_files() {
    let model = Model::load(root().join("core/tests/fixtures/tiny.kfm")).unwrap();
    let cases: Vec<TinyLog> = serde_json::from_str(include_str!("fixtures/tiny.json")).unwrap();
    for row in cases {
        assert!(
            (model.logprob(&row.run, row.complete) - row.score).abs() < 1e-4,
            "{:?}",
            row.run
        );
    }
}
#[derive(Deserialize)]
struct TinyDecide {
    config: Config,
    #[serde(flatten)]
    decision: DecideVector,
}
#[test]
fn decision_configurations_without_generic_files() {
    let cases: Vec<TinyDecide> =
        serde_json::from_str(include_str!("fixtures/decide.json")).unwrap();
    assert!(cases.len() > 100);
    for row in cases {
        let path = root().join("core/tests/fixtures/tiny.kfm");
        let detector = Detector::new(
            row.config,
            Model::load(&path).unwrap(),
            Model::load(&path).unwrap(),
            Keymap::mac_default(),
        );
        assert_decision(&detector, &row.decision);
    }
}
