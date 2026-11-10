use keyflip_core::tracker::TrackerEvent;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub enabled: bool,
    pub models_dir: Option<PathBuf>,
    pub delete_mode: String,
    pub undo_shift_tap_ms: u64,
    pub excluded_processes: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            models_dir: None,
            delete_mode: "scalar".into(),
            undo_shift_tap_ms: 400,
            excluded_processes: crate::exclude::defaults(),
            extra: Map::new(),
        }
    }
}
impl Settings {
    pub fn engine_config(&self, never: &BTreeSet<String>) -> String {
        let mut value = serde_json::to_value(self).expect("settings serialization");
        value["never"] = json!(never);
        value.to_string()
    }
}

pub type Stats = BTreeMap<String, BTreeMap<String, u64>>;
pub struct Store {
    pub root: PathBuf,
    pub settings: Settings,
    pub never: BTreeSet<String>,
    pub stats: Stats,
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}
fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(tmp, path)
}
pub fn data_root(appdata: impl AsRef<Path>) -> PathBuf {
    appdata.as_ref().join("KeyFlip")
}
impl Store {
    pub fn open(root: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&root)?;
        let settings = read_json(&root.join("settings.json")).unwrap_or_default();
        let never = fs::read_to_string(root.join("never.txt"))
            .unwrap_or_default()
            .lines()
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        let stats = read_json(&root.join("stats.json")).unwrap_or_default();
        let store = Self {
            root,
            settings,
            never,
            stats,
        };
        store.save_settings()?;
        Ok(store)
    }
    pub fn save_settings(&self) -> io::Result<()> {
        write_json(&self.root.join("settings.json"), &self.settings)
    }
    pub fn flush(&self) -> io::Result<()> {
        write_json(&self.root.join("stats.json"), &self.stats)
    }
    pub fn set_never(&mut self, never: BTreeSet<String>) -> io::Result<()> {
        if never != self.never || !self.root.join("never.txt").exists() {
            let text: String = never.iter().map(|s| format!("{s}\n")).collect();
            let tmp = self.root.join("never.tmp");
            fs::write(&tmp, text)?;
            fs::rename(tmp, self.root.join("never.txt"))?;
            self.never = never;
        }
        Ok(())
    }
    pub fn models(&self, exe_dir: &Path) -> PathBuf {
        if let Some(path) = &self.settings.models_dir {
            return path.clone();
        }
        let personal = self.root.join("models");
        if ["th.kfm", "en.kfm"]
            .iter()
            .all(|name| personal.join(name).is_file())
        {
            personal
        } else {
            exe_dir.join("models")
        }
    }
    pub fn counts(&self, day: &str) -> (u64, u64, u64) {
        let get = |key: &str| {
            self.stats
                .get(day)
                .and_then(|s| s.get(key))
                .copied()
                .unwrap_or(0)
        };
        (
            get("runs_th") + get("runs_en") + get("runs_neutral"),
            get("auto_typing") + get("auto_space"),
            get("undo_backspace") + get("undo_shift"),
        )
    }
    pub fn record(&mut self, events: Vec<TrackerEvent>, app: &str, seconds: u64) -> io::Result<()> {
        let (day, stamp) = timestamp(seconds);
        for event in events {
            let (counter, row) = match event {
                TrackerEvent::RunEnded { lang } => (format!("runs_{lang}"), None),
                TrackerEvent::Converted {
                    kind,
                    from,
                    to,
                    score,
                    key,
                    prev,
                    word,
                    result,
                } => (
                    format!("auto_{kind}"),
                    Some(
                        json!({"ts": stamp, "app": app, "kind": kind, "from": from, "to": to, "score": score, "key": key, "prev": prev, "word": word, "result": result}),
                    ),
                ),
                TrackerEvent::Reverted {
                    kind,
                    from,
                    to,
                    word,
                    result,
                } => (
                    format!("undo_{kind}"),
                    Some(
                        json!({"ts": stamp, "app": app, "kind": format!("undo_{kind}"), "from": from, "to": to, "score": 0, "key": null, "prev": null, "word": word, "result": result}),
                    ),
                ),
            };
            *self
                .stats
                .entry(day.clone())
                .or_default()
                .entry(counter)
                .or_default() += 1;
            if let Some(row) = row {
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.root.join("events.jsonl"))?;
                writeln!(file, "{row}")?;
            }
        }
        self.flush()
    }
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
/// Gregorian civil date from Unix days (UTC, matching the Mac ISO8601 formatter).
pub fn timestamp(seconds: u64) -> (String, String) {
    let z = (seconds / 86400) as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let date = format!("{year:04}-{month:02}-{day:02}");
    let t = seconds % 86400;
    (
        date.clone(),
        format!("{date}T{:02}:{:02}:{:02}Z", t / 3600, t / 60 % 60, t % 60),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyflip_core::tracker::Lang;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "keyflip-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn defaults_paths_settings_and_never_roundtrip() {
        let tmp = Temp::new();
        let root = data_root(&tmp.0);
        let mut store = Store::open(root.clone()).unwrap();
        assert!(store.settings.enabled);
        assert_eq!(store.settings.delete_mode, "scalar");
        assert_eq!(store.settings.undo_shift_tap_ms, 400);
        assert_eq!(store.settings.excluded_processes.len(), 10);
        store.settings.enabled = false;
        store.settings.extra.insert("custom".into(), json!(123));
        store.save_settings().unwrap();
        store
            .set_never(["hello".into(), "สวัสดี".into()].into())
            .unwrap();
        let mut loaded = Store::open(root).unwrap();
        assert!(!loaded.settings.enabled);
        assert_eq!(loaded.never, store.never);
        let config: Value =
            serde_json::from_str(&loaded.settings.engine_config(&loaded.never)).unwrap();
        assert_eq!(config["custom"], 123);
        assert_eq!(config["never"].as_array().unwrap().len(), 2);
        loaded.set_never(BTreeSet::new()).unwrap();
        assert_eq!(
            fs::read_to_string(loaded.root.join("never.txt")).unwrap(),
            ""
        );
    }
    #[test]
    fn partial_settings_models_and_corrupt_files() {
        let tmp = Temp::new();
        fs::write(
            tmp.0.join("settings.json"),
            r#"{"excludedProcesses":[],"enabled":false}"#,
        )
        .unwrap();
        let store = Store::open(tmp.0.clone()).unwrap();
        assert!(store.settings.excluded_processes.is_empty());
        assert_eq!(store.settings.delete_mode, "scalar");
        assert_eq!(store.models(Path::new("exe")), Path::new("exe/models"));
        fs::create_dir(tmp.0.join("models")).unwrap();
        fs::write(tmp.0.join("models/th.kfm"), "").unwrap();
        assert_eq!(store.models(Path::new("exe")), Path::new("exe/models"));
        fs::write(tmp.0.join("models/en.kfm"), "").unwrap();
        assert_eq!(store.models(Path::new("exe")), tmp.0.join("models"));
        let mut overridden = store.settings.clone();
        overridden.models_dir = Some(PathBuf::from("custom-models"));
        let custom = Store {
            root: store.root.clone(),
            settings: overridden,
            never: BTreeSet::new(),
            stats: BTreeMap::new(),
        };
        assert_eq!(custom.models(Path::new("exe")), Path::new("custom-models"));
        fs::write(tmp.0.join("settings.json"), "broken").unwrap();
        fs::write(tmp.0.join("stats.json"), "broken").unwrap();
        assert!(Store::open(tmp.0.clone()).unwrap().settings.enabled);
    }
    #[test]
    fn event_schema_daily_stats_and_no_run_text_log() {
        let tmp = Temp::new();
        let mut store = Store::open(tmp.0.clone()).unwrap();
        store
            .record(
                vec![TrackerEvent::RunEnded { lang: "th".into() }],
                "notepad.exe",
                0,
            )
            .unwrap();
        assert!(!tmp.0.join("events.jsonl").exists());
        store
            .record(
                vec![
                    TrackerEvent::Converted {
                        kind: "space".into(),
                        from: Lang::En,
                        to: Lang::Th,
                        score: 2.5,
                        key: None,
                        prev: None,
                        word: "abc".into(),
                        result: "ฟิแ".into(),
                    },
                    TrackerEvent::Reverted {
                        kind: "shift".into(),
                        from: Lang::Th,
                        to: Lang::En,
                        word: "ฟิแ".into(),
                        result: "abc".into(),
                    },
                ],
                "notepad.exe",
                0,
            )
            .unwrap();
        assert_eq!(store.counts("1970-01-01"), (1, 1, 1));
        assert_eq!(store.counts("1970-01-02"), (0, 0, 0));
        let rows: Vec<Value> = fs::read_to_string(tmp.0.join("events.jsonl"))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["ts"], "1970-01-01T00:00:00Z");
        assert_eq!(rows[0]["from"], "en");
        assert_eq!(rows[1]["kind"], "undo_shift");
        assert!(rows[1]["key"].is_null());
        assert_eq!(
            Store::open(tmp.0.clone()).unwrap().counts("1970-01-01"),
            (1, 1, 1)
        );
    }
    #[test]
    fn counters_roll_over_and_cover_all_mac_counter_names() {
        let tmp = Temp::new();
        let mut store = Store::open(tmp.0.clone()).unwrap();
        store
            .record(
                vec![
                    TrackerEvent::RunEnded { lang: "en".into() },
                    TrackerEvent::RunEnded {
                        lang: "neutral".into(),
                    },
                    TrackerEvent::Converted {
                        kind: "typing".into(),
                        from: Lang::En,
                        to: Lang::Th,
                        score: 2.0,
                        key: Some(3),
                        prev: Some(Lang::En),
                        word: "abc".into(),
                        result: "ฟิแ".into(),
                    },
                    TrackerEvent::Reverted {
                        kind: "backspace".into(),
                        from: Lang::Th,
                        to: Lang::En,
                        word: "ฟิแ".into(),
                        result: "abc".into(),
                    },
                ],
                "notepad.exe",
                86399,
            )
            .unwrap();
        store
            .record(
                vec![TrackerEvent::RunEnded { lang: "th".into() }],
                "notepad.exe",
                86400,
            )
            .unwrap();
        assert_eq!(store.counts("1970-01-01"), (2, 1, 1));
        assert_eq!(store.counts("1970-01-02"), (1, 0, 0));
        let rows: Vec<Value> = fs::read_to_string(tmp.0.join("events.jsonl"))
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(rows[0]["key"], 3);
        assert_eq!(rows[0]["prev"], "en");
        assert_eq!(rows[1]["kind"], "undo_backspace");
    }

    #[test]
    fn dates_cross_leap_day_and_midnight() {
        assert_eq!(timestamp(951782399).1, "2000-02-28T23:59:59Z");
        assert_eq!(timestamp(951782400).1, "2000-02-29T00:00:00Z");
        assert_eq!(timestamp(951868800).1, "2000-03-01T00:00:00Z");
    }
}
