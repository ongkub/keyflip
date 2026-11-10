use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
pub struct Models(pub PathBuf);
impl Models {
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "keyflip-models-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        for lang in ["th", "en"] {
            fs::write(
                path.join(format!("{lang}.kfm")),
                include_bytes!("../fixtures/tiny.kfm"),
            )
            .unwrap();
        }
        Self(path)
    }
}
impl Drop for Models {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub const CONFIG: &str = r#"{"threshold":-1000,"final_threshold":-1000,"min_len":2,"orthography_auto":false,"orthography_keep":0}"#;
pub const PAIRS: &str = r#"[["a","ก"],["b","ข"]]"#;
