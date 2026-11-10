use crate::store::{self, Store};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

pub const MAX_BODY: usize = 16 * 1024;
pub const ENDPOINT: &str = "https://keyflip.sparkth.io/v1/reports";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentWord {
    pub word: String,
    pub result: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    pub schema: u8,
    pub app: String,
    pub platform: String,
    pub app_version: String,
    pub os_version: String,
    pub model: String,
    pub enabled: bool,
    pub stats_today: StatsToday,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_words: Option<Vec<RecentWord>>,
    pub locale: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatsToday {
    pub runs: u64,
    pub conversions: u64,
    pub undos: u64,
}

fn cap_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}
pub fn recent_words(path: &Path) -> Vec<RecentWord> {
    let Ok(contents) = fs::read_to_string(path) else {
        return vec![];
    };
    contents
        .lines()
        .filter_map(|line| {
            let row: Value = serde_json::from_str(line).ok()?;
            let word = row.get("word")?.as_str()?;
            let result = row.get("result")?.as_str()?;
            let raw_kind = row.get("kind")?.as_str()?;
            let kind = if raw_kind.starts_with("undo_") {
                "undo"
            } else if raw_kind.starts_with("auto_") {
                "typing"
            } else {
                return None;
            };
            Some(RecentWord {
                word: cap_chars(word, 40),
                result: cap_chars(result, 40),
                kind: kind.into(),
            })
        })
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
pub fn build(
    store: &Store,
    os_version: &str,
    model: &str,
    message: &str,
    contact: &str,
    attach: bool,
) -> Report {
    let counts = store.counts(&store::timestamp(store::now()).0);
    Report {
        schema: 1,
        app: "keyflip".into(),
        platform: "windows".into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        os_version: cap_chars(os_version, 100),
        model: model.into(),
        enabled: store.settings.enabled,
        stats_today: StatsToday {
            runs: counts.0,
            conversions: counts.1,
            undos: counts.2,
        },
        message: message.to_owned(),
        contact: if contact.trim().is_empty() {
            None
        } else {
            Some(contact.to_owned())
        },
        recent_words: attach.then(|| recent_words(&store.root.join("events.jsonl"))),
        locale: "th-TH".into(),
    }
}
pub fn validate(report: &Report) -> Result<Vec<u8>, String> {
    if report.message.trim().is_empty() || report.message.chars().count() > 2000 {
        return Err("กรุณาเขียนรายละเอียดปัญหา (ไม่เกิน 2,000 ตัวอักษร)".into());
    }
    if report
        .contact
        .as_ref()
        .is_some_and(|s| s.chars().count() > 200)
    {
        return Err("ข้อมูลติดต่อยาวเกิน 200 ตัวอักษร".into());
    }
    if let Some(words) = &report.recent_words {
        if words.len() > 10
            || words
                .iter()
                .any(|w| w.word.chars().count() > 40 || w.result.chars().count() > 40)
        {
            return Err("ข้อมูลคำล่าสุดไม่ถูกต้อง".into());
        }
    }
    let body = serde_json::to_vec(report).map_err(|e| e.to_string())?;
    if body.len() > MAX_BODY {
        return Err("รายงานมีขนาดใหญ่เกิน 16 KB".into());
    }
    Ok(body)
}
pub fn response_message(status: u32, body: &[u8]) -> String {
    match status {
        201 => "ส่งรายงานแล้ว ขอบคุณที่ช่วยแจ้งปัญหา".into(),
        400 => "ข้อมูลรายงานไม่ถูกต้อง กรุณาตรวจข้อความแล้วลองอีกครั้ง".into(),
        413 => "รายงานมีขนาดใหญ่เกินกำหนด กรุณาลดข้อมูลแล้วลองอีกครั้ง".into(),
        429 => "ส่งรายงานบ่อยเกินไป กรุณารอสักครู่แล้วลองอีกครั้ง".into(),
        _ => serde_json::from_slice::<Value>(body)
            .ok()
            .and_then(|v| v.get("error")?.as_str().map(str::to_owned))
            .map(|e| format!("ส่งรายงานไม่สำเร็จ: {e}"))
            .unwrap_or_else(|| format!("ส่งรายงานไม่สำเร็จ (HTTP {status})")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Settings, Stats};
    use std::{collections::BTreeSet, path::PathBuf};
    fn store(root: PathBuf) -> Store {
        Store {
            root,
            settings: Settings::default(),
            never: BTreeSet::new(),
            stats: Stats::new(),
        }
    }
    #[test]
    fn build_opt_in_and_caps_words() {
        let root = std::env::temp_dir().join(format!("keyflip-report-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("events.jsonl"),
            format!(
                "{{\"word\":\"{}\",\"result\":\"result\",\"kind\":\"auto_typing\"}}\n",
                "x".repeat(60)
            ),
        )
        .unwrap();
        let st = store(root.clone());
        let plain = build(&st, "Windows 11", "generic", "broken", "", false);
        assert!(plain.recent_words.is_none());
        assert!(plain.contact.is_none());
        let with = build(&st, "Windows 11", "generic", "broken", "me", true);
        assert_eq!(
            with.recent_words.as_ref().unwrap()[0].word.chars().count(),
            40
        );
        assert_eq!(with.recent_words.as_ref().unwrap()[0].kind, "typing");
        assert_eq!(
            serde_json::from_slice::<Value>(&validate(&with).unwrap()).unwrap()["platform"],
            "windows"
        );
        let _ = fs::remove_dir_all(root);
    }
    #[test]
    fn validates_and_maps_responses() {
        let st = store(PathBuf::from("."));
        let mut report = build(&st, "Windows", "open", " ", "", false);
        assert!(validate(&report).is_err());
        report.message = "ปัญหา".into();
        assert!(validate(&report).is_ok());
        assert!(validate(&build(&st, "Windows", "open", &"x".repeat(2001), "", false)).is_err());
        assert!(validate(&build(
            &st,
            "Windows",
            "open",
            "valid",
            &"x".repeat(201),
            false
        ))
        .is_err());
        assert!(response_message(201, br#"{"ok":true,"id":4}"#).contains("ส่งรายงานแล้ว"));
        assert!(response_message(400, b"").contains("ไม่ถูกต้อง"));
        assert!(response_message(413, b"").contains("ใหญ่เกิน"));
        assert!(response_message(429, b"").contains("รอสักครู่"));
    }
}
