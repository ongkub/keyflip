//! Model/configuration loading and the portable per-keystroke engine.
//!
//! Configuration is one JSON object containing the detector's snake_case keys
//! plus `deleteMode` (`scalar`, the default, or `grapheme`) and `never` (an optional
//! array of words; `initialNever`/`neverList` are aliases). Absent detector keys
//! use the tuned `detector/config.json` (embedded at build time), not the Python
//! constructor defaults. Pair tables are `[[abc, thai], ...]`.
use crate::{
    detect::{Config, Detector},
    keymap::Keymap,
    model::Model,
    tracker::{Action, Key, Mapper, RunTracker, TrackerEvent},
};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

#[derive(Deserialize)]
struct EngineConfig {
    #[serde(flatten)]
    detector: Config,
    #[serde(default = "scalar", rename = "deleteMode")]
    delete_mode: String,
    #[serde(default, alias = "initialNever", alias = "neverList")]
    never: BTreeSet<String>,
}
fn scalar() -> String {
    "scalar".into()
}

/// The thresholds tuned on held-out chats (see detector/RESULTS.md, v3); user keys override them.
const TUNED_CONFIG: &str = include_str!("../../detector/config.json");

/// Owns both the detector and tracker (the detector is stored inside the tracker).
/// Calls on an engine must be serialized; the C wrapper is single-threaded.
pub struct Engine {
    tracker: RunTracker<Detector>,
}
impl Engine {
    pub fn new(
        models_dir: impl AsRef<Path>,
        config_json: Option<&str>,
        pairs_json: Option<&str>,
    ) -> Result<Self, String> {
        // Start from the tuned shipping config so a shell that passes nothing (or only a few
        // keys) never falls back to the untuned Python constructor defaults.
        let mut merged: serde_json::Value =
            serde_json::from_str(TUNED_CONFIG).expect("tuned config is valid JSON");
        let user: serde_json::Value = serde_json::from_str(config_json.unwrap_or("{}"))
            .map_err(|e| format!("invalid config JSON: {e}"))?;
        match (merged.as_object_mut(), user) {
            (Some(base), serde_json::Value::Object(over)) => base.extend(over),
            _ => return Err("config JSON must be an object".into()),
        }
        let config: EngineConfig =
            serde_json::from_value(merged).map_err(|e| format!("invalid config JSON: {e}"))?;
        if !matches!(config.delete_mode.as_str(), "scalar" | "grapheme") {
            return Err("deleteMode must be scalar or grapheme".into());
        }
        let keymap = if let Some(json) = pairs_json {
            let pairs: Vec<(String, String)> =
                serde_json::from_str(json).map_err(|e| format!("invalid key-pair JSON: {e}"))?;
            if pairs
                .iter()
                .any(|(a, b)| a.contains('\0') || b.contains('\0'))
            {
                return Err("key pairs must not contain NUL".into());
            }
            Keymap::from_pairs(&pairs)
        } else {
            Keymap::mac_default()
        };
        let detector = Detector::new(
            config.detector,
            Model::load(models_dir.as_ref().join("th.kfm"))?,
            Model::load(models_dir.as_ref().join("en.kfm"))?,
            keymap.clone(),
        );
        Ok(Self {
            tracker: RunTracker::new(
                detector,
                Mapper::new(keymap),
                config.never,
                &config.delete_mode,
            ),
        })
    }
    pub fn detector(&self) -> &Detector {
        self.tracker.decider()
    }
    pub fn handle(&mut self, key: Key) -> Action {
        self.tracker.handle(key)
    }
    pub fn take_events(&mut self) -> Vec<TrackerEvent> {
        self.tracker.take_events()
    }
    pub fn never(&self) -> &BTreeSet<String> {
        self.tracker.never()
    }
    pub fn set_never(&mut self, never: BTreeSet<String>) {
        self.tracker.set_never(never);
    }
    pub fn clear_never(&mut self) {
        self.tracker.clear_never();
    }
}
