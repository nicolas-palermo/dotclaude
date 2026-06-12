use crate::data::model::{Feature, FeaturesFile, Handoff, MissionState, ProgressEvent};
use std::fs;
use std::path::Path;

/// An immutable snapshot of all mission data loaded from a mission directory.
#[derive(Debug, Clone, Default)]
pub struct MissionSnapshot {
    pub state: MissionState,
    pub features: Vec<Feature>,
    pub progress_events: Vec<ProgressEvent>,
    pub handoffs: Vec<Handoff>,
    /// Non-fatal warnings collected during loading (corrupt lines, unknown fields, etc.)
    pub warnings: Vec<String>,
}

impl MissionSnapshot {
    /// Load a snapshot from a mission directory. Never panics on missing or corrupt files.
    pub fn load(mission_dir: &Path) -> MissionSnapshot {
        let mut snap = MissionSnapshot::default();

        // state.json
        let state_path = mission_dir.join("state.json");
        match fs::read_to_string(&state_path) {
            Err(_) => {
                // Missing file is normal; leave state as default.
            }
            Ok(text) => match serde_json::from_str::<MissionState>(&text) {
                Ok(s) => snap.state = s,
                Err(e) => snap.warnings.push(format!("state.json parse error: {e}")),
            },
        }

        // features.json
        let features_path = mission_dir.join("features.json");
        match fs::read_to_string(&features_path) {
            Err(_) => {}
            Ok(text) => match serde_json::from_str::<FeaturesFile>(&text) {
                Ok(f) => snap.features = f.features,
                Err(e) => snap
                    .warnings
                    .push(format!("features.json parse error: {e}")),
            },
        }

        // progress_log.jsonl — line-by-line, bad lines skipped with a warning
        let log_path = mission_dir.join("progress_log.jsonl");
        if let Ok(text) = fs::read_to_string(&log_path) {
            for (i, line) in text.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ProgressEvent>(trimmed) {
                    Ok(ev) => snap.progress_events.push(ev),
                    Err(e) => snap
                        .warnings
                        .push(format!("progress_log.jsonl line {}: {e}", i + 1)),
                }
            }
        }

        // handoffs/*.json
        let handoffs_dir = mission_dir.join("handoffs");
        if let Ok(entries) = fs::read_dir(&handoffs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                match fs::read_to_string(&path) {
                    Err(e) => snap
                        .warnings
                        .push(format!("handoff {} unreadable: {e}", path.display())),
                    Ok(text) => match serde_json::from_str::<Handoff>(&text) {
                        Ok(h) => snap.handoffs.push(h),
                        Err(e) => snap
                            .warnings
                            .push(format!("handoff {} parse error: {e}", path.display())),
                    },
                }
            }
        }

        snap
    }
}
