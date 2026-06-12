use crate::data::loader::MissionSnapshot;
use crate::data::model::Feature;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// High-level summary derived from a `MissionSnapshot`.
#[derive(Debug, Clone)]
pub struct MissionSummary {
    pub mission_id: String,
    pub state: String,
    pub milestone: Option<String>,
    pub total_features: usize,
    pub completed_features: usize,
    /// Feature currently `in_progress`, if any.
    pub active_feature: Option<Feature>,
    /// Elapsed time since `createdAt` (computed with injectable `now`).
    pub elapsed_secs: Option<i64>,
}

impl MissionSummary {
    /// Derive from a snapshot using the given `now` timestamp (injectable for tests).
    pub fn from_snapshot(snap: &MissionSnapshot, now: DateTime<Utc>) -> MissionSummary {
        let total = snap.features.len();
        let completed = snap
            .features
            .iter()
            .filter(|f| f.status == "completed")
            .count();
        let active = snap
            .features
            .iter()
            .find(|f| f.status == "in_progress")
            .cloned();

        let elapsed = snap
            .state
            .created_at
            .map(|created| (now - created).num_seconds());

        MissionSummary {
            mission_id: snap.state.mission_id.clone(),
            state: snap.state.state.clone(),
            milestone: snap.state.current_milestone.clone(),
            total_features: total,
            completed_features: completed,
            active_feature: active,
            elapsed_secs: elapsed,
        }
    }
}

/// Status of a worker session as derived from progress events and handoffs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Running,
    Success,
    Failed,
    Partial,
}

impl WorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerStatus::Running => "Running",
            WorkerStatus::Success => "Success",
            WorkerStatus::Failed => "Failed",
            WorkerStatus::Partial => "Partial",
        }
    }
}

/// A derived worker session.
#[derive(Debug, Clone)]
pub struct WorkerSession {
    /// 1-based ordinal across all sessions (sorted by start time).
    pub ordinal: usize,
    pub session_id: String,
    pub feature_id: Option<String>,
    pub start: DateTime<Utc>,
    /// Duration in seconds. None if still running and start is unknown.
    pub duration_secs: Option<i64>,
    pub status: WorkerStatus,
}

/// Derive worker sessions from a snapshot using the given `now` for running duration.
pub fn derive_worker_sessions(snap: &MissionSnapshot, now: DateTime<Utc>) -> Vec<WorkerSession> {
    // Keyed by workerSessionId; we track start, end, feature, and success_state from handoffs.
    #[derive(Default)]
    struct Acc {
        feature_id: Option<String>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        success_state: Option<String>,
    }

    let mut map: HashMap<String, Acc> = HashMap::new();

    for ev in &snap.progress_events {
        let Some(sid) = &ev.worker_session_id else {
            continue;
        };
        let acc = map.entry(sid.clone()).or_default();

        match ev.event_type.as_str() {
            "worker_started" => {
                if acc.start.is_none() {
                    acc.start = Some(ev.timestamp);
                }
                if acc.feature_id.is_none() {
                    acc.feature_id = ev.feature_id.clone();
                }
            }
            "worker_selected_feature" => {
                if acc.feature_id.is_none() {
                    acc.feature_id = ev.feature_id.clone();
                }
            }
            "worker_completed" | "worker_failed" | "worker_abnormal_stop" => {
                acc.end = Some(ev.timestamp);
                if acc.feature_id.is_none() {
                    acc.feature_id = ev.feature_id.clone();
                }
                if let Some(ss) = &ev.success_state {
                    acc.success_state = Some(ss.clone());
                }
            }
            _ => {}
        }
    }

    // Enrich with handoff successState (authoritative).
    for handoff in &snap.handoffs {
        let acc = map.entry(handoff.worker_session_id.clone()).or_default();
        acc.success_state = Some(handoff.success_state.clone());
        if acc.feature_id.is_none() {
            acc.feature_id = Some(handoff.feature_id.clone());
        }
    }

    // Build sessions and sort by start time.
    let mut sessions: Vec<(DateTime<Utc>, WorkerSession)> = map
        .into_iter()
        .map(|(sid, acc)| {
            let start = acc.start.unwrap_or(now);
            let status = match acc.end {
                None => WorkerStatus::Running,
                Some(_) => match acc.success_state.as_deref() {
                    Some("success") => WorkerStatus::Success,
                    Some("failure") => WorkerStatus::Failed,
                    Some("partial") => WorkerStatus::Partial,
                    _ => WorkerStatus::Failed,
                },
            };
            let duration_secs = match &status {
                WorkerStatus::Running => Some((now - start).num_seconds()),
                _ => acc.end.map(|end| (end - start).num_seconds()),
            };
            (
                start,
                WorkerSession {
                    ordinal: 0, // assigned below
                    session_id: sid,
                    feature_id: acc.feature_id,
                    start,
                    duration_secs,
                    status,
                },
            )
        })
        .collect();

    sessions.sort_by_key(|(start, _)| *start);

    sessions
        .into_iter()
        .enumerate()
        .map(|(i, (_, mut ws))| {
            ws.ordinal = i + 1;
            ws
        })
        .collect()
}
