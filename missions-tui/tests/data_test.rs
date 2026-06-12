/// Integration tests for the data layer (model + loader + derive).
///
/// VAL-DATA-001: Realistic fixtures parse into typed models with correct field values.
/// VAL-DATA-002: Missing/corrupt files yield empty defaults + warnings, never a panic.
use chrono::Utc;
use missions_tui::data::{derive, loader::MissionSnapshot};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn mission_a_dir() -> PathBuf {
    fixtures_dir().join("mission-a")
}

fn mission_empty_dir() -> PathBuf {
    fixtures_dir().join("mission-empty")
}

fn mission_corrupt_dir() -> PathBuf {
    fixtures_dir().join("mission-corrupt")
}

// ---------------------------------------------------------------------------
// VAL-DATA-001: Realistic fixtures parse correctly
// ---------------------------------------------------------------------------

#[test]
fn val_data_001_state_fields_parse_correctly() {
    // verifies: "VAL-DATA-001: state.json fields map into MissionState with correct values"
    let snap = MissionSnapshot::load(&mission_a_dir());

    assert_eq!(
        snap.state.mission_id, "99038ad5-abbe-4534-9e5a-d14ece4274e6",
        "missionId should match fixture"
    );
    assert_eq!(snap.state.state, "running", "state should be 'running'");
    assert_eq!(
        snap.state.working_directory,
        "/Users/galina/orca/workspaces/dotclaude/missions-tui"
    );
    assert_eq!(
        snap.state.current_milestone.as_deref(),
        Some("m1-data-layer"),
        "currentMilestone should be 'm1-data-layer'"
    );
    assert!(
        snap.state.created_at.is_some(),
        "createdAt should parse as a timestamp"
    );
    assert!(
        snap.state.updated_at.is_some(),
        "updatedAt should parse as a timestamp"
    );
    // No warnings expected for well-formed state.json
    let state_warnings: Vec<_> = snap
        .warnings
        .iter()
        .filter(|w| w.contains("state.json"))
        .collect();
    assert!(
        state_warnings.is_empty(),
        "no warnings expected for state.json; got: {state_warnings:?}"
    );
}

#[test]
fn val_data_001_features_parse_correctly() {
    // verifies: "VAL-DATA-001: features.json parses into Feature structs with correct status + fulfills"
    let snap = MissionSnapshot::load(&mission_a_dir());

    // At least 2 features: m1-registry-cli (completed) and m1-mission-data-model (in_progress)
    assert!(
        snap.features.len() >= 2,
        "expected at least 2 features, got {}",
        snap.features.len()
    );

    let registry_feat = snap
        .features
        .iter()
        .find(|f| f.id == "m1-registry-cli")
        .expect("m1-registry-cli feature must be present");
    assert_eq!(
        registry_feat.status, "completed",
        "m1-registry-cli should be completed"
    );
    assert_eq!(
        registry_feat.milestone, "m1-data-layer",
        "milestone should be m1-data-layer"
    );
    assert!(
        registry_feat.fulfills.contains(&"VAL-REG-001".to_string()),
        "m1-registry-cli.fulfills must include VAL-REG-001"
    );
    assert_eq!(
        registry_feat.completed_worker_session_id.as_deref(),
        Some("77658ccc-e4f3-4b4d-b734-b75e99c7236c"),
        "completedWorkerSessionId must be set for completed feature"
    );

    let data_feat = snap
        .features
        .iter()
        .find(|f| f.id == "m1-mission-data-model")
        .expect("m1-mission-data-model feature must be present");
    assert_eq!(
        data_feat.status, "in_progress",
        "m1-mission-data-model should be in_progress"
    );
    assert!(
        data_feat.fulfills.contains(&"VAL-DATA-001".to_string()),
        "m1-mission-data-model.fulfills must include VAL-DATA-001"
    );
    assert_eq!(
        data_feat.current_worker_session_id.as_deref(),
        Some("8bf19191-54bc-4fef-a079-53b33706cf7b"),
        "currentWorkerSessionId must be set for in_progress feature"
    );
    assert!(
        data_feat.completed_worker_session_id.is_none(),
        "completedWorkerSessionId must be None for in_progress feature"
    );
}

#[test]
fn val_data_001_progress_events_parse_correctly() {
    // verifies: "VAL-DATA-001: progress_log.jsonl lines parse into ProgressEvent structs"
    let snap = MissionSnapshot::load(&mission_a_dir());

    // All 8 lines are valid, so no progress-log warnings expected
    let log_warnings: Vec<_> = snap
        .warnings
        .iter()
        .filter(|w| w.contains("progress_log"))
        .collect();
    assert!(
        log_warnings.is_empty(),
        "no warnings expected for valid progress_log; got: {log_warnings:?}"
    );

    // Should have at least 8 events (the 8 lines we wrote)
    assert!(
        snap.progress_events.len() >= 8,
        "expected at least 8 progress events, got {}",
        snap.progress_events.len()
    );

    // Verify the still-running worker session event is present
    let running_started = snap.progress_events.iter().find(|ev| {
        ev.event_type == "worker_started"
            && ev.worker_session_id.as_deref() == Some("8bf19191-54bc-4fef-a079-53b33706cf7b")
    });
    assert!(
        running_started.is_some(),
        "worker_started event for the still-running session must be present"
    );

    // Verify there is NO worker_completed for the running session
    let running_completed = snap.progress_events.iter().find(|ev| {
        (ev.event_type == "worker_completed" || ev.event_type == "worker_failed")
            && ev.worker_session_id.as_deref() == Some("8bf19191-54bc-4fef-a079-53b33706cf7b")
    });
    assert!(
        running_completed.is_none(),
        "no completed event expected for the still-running session"
    );
}

#[test]
fn val_data_001_handoffs_parse_correctly() {
    // verifies: "VAL-DATA-001: handoffs/*.json parse into Handoff structs with successState + commitId"
    let snap = MissionSnapshot::load(&mission_a_dir());

    // Should have at least 2 handoffs: prior-example.json + m1-registry-cli.json
    assert!(
        snap.handoffs.len() >= 2,
        "expected at least 2 handoffs, got {}",
        snap.handoffs.len()
    );

    let registry_handoff = snap
        .handoffs
        .iter()
        .find(|h| h.feature_id == "m1-registry-cli")
        .expect("m1-registry-cli handoff must be present");
    assert_eq!(
        registry_handoff.success_state, "success",
        "successState should be 'success'"
    );
    assert_eq!(
        registry_handoff.milestone, "m1-data-layer",
        "milestone should match"
    );
    assert_eq!(
        registry_handoff.worker_session_id,
        "77658ccc-e4f3-4b4d-b734-b75e99c7236c"
    );
    assert_eq!(
        registry_handoff.commit_id.as_deref(),
        Some("581515d"),
        "commitId should be set"
    );

    // No handoff-parse warnings for mission-a
    let handoff_warnings: Vec<_> = snap
        .warnings
        .iter()
        .filter(|w| w.contains("handoff") && w.contains("parse error"))
        .collect();
    assert!(
        handoff_warnings.is_empty(),
        "no parse errors expected for valid handoffs; got: {handoff_warnings:?}"
    );
}

#[test]
fn val_data_001_timestamps_parse_as_utc() {
    // verifies: "VAL-DATA-001: ISO-8601 timestamps in fixtures parse as chrono::DateTime<Utc>"
    let snap = MissionSnapshot::load(&mission_a_dir());

    let created = snap
        .state
        .created_at
        .expect("createdAt must parse as DateTime<Utc>");
    let updated = snap
        .state
        .updated_at
        .expect("updatedAt must parse as DateTime<Utc>");
    assert!(
        updated > created,
        "updatedAt should be after createdAt; created={created}, updated={updated}"
    );

    // Spot-check a progress event timestamp
    let first_ev = &snap.progress_events[0];
    // 2026-06-11T23:21:33Z → seconds since epoch must be positive
    assert!(
        first_ev.timestamp.timestamp() > 0,
        "progress event timestamp must be valid"
    );
}

#[test]
fn val_data_001_derived_mission_summary() {
    // verifies: "VAL-DATA-001: MissionSummary derived from realistic snapshot has correct counts"
    let snap = MissionSnapshot::load(&mission_a_dir());
    // Use a fixed "now" well after fixture timestamps
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    assert_eq!(summary.mission_id, "99038ad5-abbe-4534-9e5a-d14ece4274e6");
    assert_eq!(summary.state, "running");
    assert_eq!(summary.milestone.as_deref(), Some("m1-data-layer"));

    // 1 completed feature (m1-registry-cli), at least 1 in_progress
    assert!(
        summary.completed_features >= 1,
        "at least 1 completed feature expected"
    );
    assert!(
        summary.total_features >= 2,
        "at least 2 total features expected"
    );

    // Active feature should be m1-mission-data-model
    let active = summary.active_feature.expect("active_feature must be Some");
    assert_eq!(active.id, "m1-mission-data-model");

    // elapsed_secs should be positive (fixture createdAt is in 2026-06-11, now is 2026-06-12)
    let elapsed = summary.elapsed_secs.expect("elapsed_secs must be Some");
    assert!(
        elapsed > 0,
        "elapsed_secs should be positive, got {elapsed}"
    );
}

#[test]
fn val_data_001_derived_worker_sessions() {
    // verifies: "VAL-DATA-001: derive_worker_sessions produces running + success sessions from fixture"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    // Should have at least 2 sessions
    assert!(
        sessions.len() >= 2,
        "expected at least 2 worker sessions, got {}",
        sessions.len()
    );

    // The completed session (77658ccc...) should have status=Success
    let completed_sess = sessions
        .iter()
        .find(|s| s.session_id == "77658ccc-e4f3-4b4d-b734-b75e99c7236c")
        .expect("completed worker session must be present");
    assert_eq!(
        completed_sess.status,
        derive::WorkerStatus::Success,
        "worker 77658ccc should have status Success"
    );
    assert_eq!(
        completed_sess.feature_id.as_deref(),
        Some("m1-registry-cli"),
        "completed session should reference m1-registry-cli"
    );

    // The running session (8bf19191...) should have status=Running
    let running_sess = sessions
        .iter()
        .find(|s| s.session_id == "8bf19191-54bc-4fef-a079-53b33706cf7b")
        .expect("running worker session must be present");
    assert_eq!(
        running_sess.status,
        derive::WorkerStatus::Running,
        "worker 8bf19191 should have status Running (no completed event)"
    );
    assert_eq!(
        running_sess.feature_id.as_deref(),
        Some("m1-mission-data-model"),
        "running session should reference m1-mission-data-model"
    );
    // Running session should have positive duration_secs
    let duration = running_sess
        .duration_secs
        .expect("running session must have duration_secs");
    assert!(duration > 0, "running session duration should be positive");
}

// ---------------------------------------------------------------------------
// VAL-DATA-002: Degraded inputs — empty dir
// ---------------------------------------------------------------------------

#[test]
fn val_data_002_empty_dir_yields_defaults_no_panic() {
    // verifies: "VAL-DATA-002: Missing files yield empty defaults + no panic"
    let snap = MissionSnapshot::load(&mission_empty_dir());

    // Empty defaults — no crash
    assert_eq!(
        snap.state.mission_id, "",
        "mission_id should be empty string"
    );
    assert_eq!(snap.state.state, "", "state should be empty string");
    assert!(snap.features.is_empty(), "features should be empty");
    assert!(
        snap.progress_events.is_empty(),
        "progress_events should be empty"
    );
    assert!(snap.handoffs.is_empty(), "handoffs should be empty");
    // No warnings from an empty directory (missing files are not warnings)
    assert!(
        snap.warnings.is_empty(),
        "no warnings expected for missing files; got: {:?}",
        snap.warnings
    );
}

#[test]
fn val_data_002_empty_dir_derive_does_not_panic() {
    // verifies: "VAL-DATA-002: Derive functions run without panic on empty snapshot"
    let snap = MissionSnapshot::load(&mission_empty_dir());
    let now = Utc::now();
    let summary = derive::MissionSummary::from_snapshot(&snap, now);
    let sessions = derive::derive_worker_sessions(&snap, now);

    // All zero counts, no active feature
    assert_eq!(summary.total_features, 0);
    assert_eq!(summary.completed_features, 0);
    assert!(summary.active_feature.is_none());
    assert!(sessions.is_empty());
}

// ---------------------------------------------------------------------------
// VAL-DATA-002: Degraded inputs — corrupt files
// ---------------------------------------------------------------------------

#[test]
fn val_data_002_corrupt_state_yields_warning_no_panic() {
    // verifies: "VAL-DATA-002: Corrupt state.json emits a warning, defaults are used, no panic"
    let snap = MissionSnapshot::load(&mission_corrupt_dir());

    // state.json is corrupt, so mission_id should be the default empty string
    assert_eq!(
        snap.state.mission_id, "",
        "corrupt state.json should yield default empty mission_id"
    );

    // A warning about state.json parse error should be present
    let has_state_warning = snap.warnings.iter().any(|w| w.contains("state.json"));
    assert!(
        has_state_warning,
        "expected a state.json warning; warnings: {:?}",
        snap.warnings
    );
}

#[test]
fn val_data_002_corrupt_jsonl_skips_bad_lines_collects_warnings() {
    // verifies: "VAL-DATA-002: Corrupt JSONL lines are skipped with warnings; good lines still parse"
    let snap = MissionSnapshot::load(&mission_corrupt_dir());

    // progress_log.jsonl has 5 lines: 2 valid, 3 corrupt
    // Valid: line 1 (mission_accepted), line 3 (mission_run_started), line 5 (worker_started)
    let good_events = snap.progress_events.len();
    assert!(
        good_events >= 2,
        "at least 2 good lines should parse; got {good_events}"
    );
    assert!(
        good_events < 5,
        "should not have parsed all 5 lines (3 are corrupt); got {good_events}"
    );

    // Should have warnings for the corrupt lines
    let log_warnings: Vec<_> = snap
        .warnings
        .iter()
        .filter(|w| w.contains("progress_log.jsonl"))
        .collect();
    assert!(
        !log_warnings.is_empty(),
        "expected warnings for corrupt JSONL lines; got none. All warnings: {:?}",
        snap.warnings
    );
    // At least 2 corrupt lines (lines 2 and 4)
    assert!(
        log_warnings.len() >= 2,
        "expected at least 2 JSONL warnings; got {}",
        log_warnings.len()
    );
}

#[test]
fn val_data_002_corrupt_handoff_yields_warning_no_panic() {
    // verifies: "VAL-DATA-002: Corrupt handoff JSON is skipped with a warning, no panic"
    let snap = MissionSnapshot::load(&mission_corrupt_dir());

    // bad-handoff.json is truncated, should produce a warning
    let has_handoff_warning = snap
        .warnings
        .iter()
        .any(|w| w.contains("handoff") || w.contains("bad-handoff"));
    assert!(
        has_handoff_warning,
        "expected a handoff parse warning; warnings: {:?}",
        snap.warnings
    );

    // Should have no valid handoffs from this dir (the only handoff is corrupt)
    assert!(
        snap.handoffs.is_empty(),
        "corrupt handoff should not appear in handoffs list"
    );
}

#[test]
fn val_data_002_corrupt_fixture_derive_does_not_panic() {
    // verifies: "VAL-DATA-002: Derive functions handle partial data from corrupt fixtures without panic"
    let snap = MissionSnapshot::load(&mission_corrupt_dir());
    let now = Utc::now();

    // These must not panic even with partial state
    let summary = derive::MissionSummary::from_snapshot(&snap, now);
    let sessions = derive::derive_worker_sessions(&snap, now);

    // state.json failed, so mission_id is default; state is empty string — that's fine
    assert_eq!(summary.mission_id, "");

    // The good progress events may contain worker_started → Running session
    // We just assert no panic and sanity-check types
    let _ = sessions.len(); // must not panic
    let _ = summary.total_features; // must not panic
}

#[test]
fn val_data_002_nonexistent_dir_no_panic() {
    // verifies: "VAL-DATA-002: Completely nonexistent mission dir produces empty snapshot + no panic"
    let nonexistent = fixtures_dir().join("this-dir-does-not-exist-xyz");
    let snap = MissionSnapshot::load(&nonexistent);

    assert_eq!(snap.state.mission_id, "");
    assert!(snap.features.is_empty());
    assert!(snap.progress_events.is_empty());
    assert!(snap.handoffs.is_empty());
    // No warnings expected — a missing directory is treated the same as missing files
    assert!(
        snap.warnings.is_empty(),
        "no warnings expected for nonexistent dir; got: {:?}",
        snap.warnings
    );
}
