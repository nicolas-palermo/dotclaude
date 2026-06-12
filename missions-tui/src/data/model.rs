use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Parsed `state.json`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MissionState {
    pub mission_id: String,
    /// Raw state string — unknown values are kept as-is and rendered verbatim.
    pub state: String,
    pub working_directory: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub current_milestone: Option<String>,
}

/// A single feature from `features.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub id: String,
    pub description: String,
    pub milestone: String,
    /// Raw status string — spec: pending | in_progress | completed | failed
    pub status: String,
    #[serde(default)]
    pub skill_name: Option<String>,
    #[serde(default)]
    pub fulfills: Vec<String>,
    #[serde(default)]
    pub preconditions: Vec<String>,
    #[serde(default)]
    pub expected_behavior: Vec<String>,
    #[serde(default)]
    pub worker_session_ids: Vec<String>,
    #[serde(default)]
    pub current_worker_session_id: Option<String>,
    #[serde(default)]
    pub completed_worker_session_id: Option<String>,
}

/// Wrapper for the features file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeaturesFile {
    pub features: Vec<Feature>,
}

/// A single line in `progress_log.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub worker_session_id: Option<String>,
    #[serde(default)]
    pub feature_id: Option<String>,
    #[serde(default)]
    pub success_state: Option<String>,
    #[serde(default)]
    pub return_to_orchestrator: Option<bool>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub milestone: Option<String>,
    // Catch-all for unknown fields (lenient parsing)
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A parsed handoff file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Handoff {
    pub timestamp: DateTime<Utc>,
    pub worker_session_id: String,
    pub feature_id: String,
    pub milestone: String,
    pub success_state: String,
    #[serde(default)]
    pub return_to_orchestrator: bool,
    #[serde(default)]
    pub commit_id: Option<String>,
    // Catch-all for unknown fields (lenient parsing)
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
