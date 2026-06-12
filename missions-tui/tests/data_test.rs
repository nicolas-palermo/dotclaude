/// Integration tests for the data layer (model + loader + derive).
///
/// VAL-DATA-001: Realistic fixtures parse into typed models with correct field values.
/// VAL-DATA-002: Missing/corrupt files yield empty defaults + warnings, never a panic.
/// VAL-DATA-003: Worker session derivation is deterministic across ticks.
/// VAL-WORK-001: Workers panel ordering is stable (newest-first display requires stable sort).
use chrono::Utc;
use missions_tui::data::model::ProgressEvent;
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
// VAL-DATA-003: Worker session derivation from progress events + handoffs
// ---------------------------------------------------------------------------

#[test]
fn val_data_003_sessions_have_correct_status() {
    // verifies: "VAL-DATA-003: derive_worker_sessions maps progress events to Running/Success status"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    // Completed session (77658ccc) must have Success status (enriched by handoff)
    let completed = sessions
        .iter()
        .find(|s| s.session_id == "77658ccc-e4f3-4b4d-b734-b75e99c7236c")
        .expect("completed worker session 77658ccc must be present");
    assert_eq!(
        completed.status,
        derive::WorkerStatus::Success,
        "77658ccc: handoff successState=success should yield WorkerStatus::Success"
    );

    // Still-running session (8bf19191) must have Running status (no end event)
    let running = sessions
        .iter()
        .find(|s| s.session_id == "8bf19191-54bc-4fef-a079-53b33706cf7b")
        .expect("running worker session 8bf19191 must be present");
    assert_eq!(
        running.status,
        derive::WorkerStatus::Running,
        "8bf19191: no worker_completed event should yield WorkerStatus::Running"
    );
}

#[test]
fn val_data_003_sessions_have_correct_feature_ids() {
    // verifies: "VAL-DATA-003: each WorkerSession carries the featureId it was assigned to"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    let completed = sessions
        .iter()
        .find(|s| s.session_id == "77658ccc-e4f3-4b4d-b734-b75e99c7236c")
        .unwrap();
    assert_eq!(
        completed.feature_id.as_deref(),
        Some("m1-registry-cli"),
        "completed session must reference m1-registry-cli"
    );

    let running = sessions
        .iter()
        .find(|s| s.session_id == "8bf19191-54bc-4fef-a079-53b33706cf7b")
        .unwrap();
    assert_eq!(
        running.feature_id.as_deref(),
        Some("m1-mission-data-model"),
        "running session must reference m1-mission-data-model"
    );
}

#[test]
fn val_data_003_sessions_have_1based_ordinals_sorted_by_start() {
    // verifies: "VAL-DATA-003: WorkerSessions are sorted by start time and assigned 1-based ordinals"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    assert!(
        sessions.len() >= 2,
        "expected at least 2 sessions for ordinal test"
    );

    // Ordinals must be 1-based and strictly increasing
    for (idx, ws) in sessions.iter().enumerate() {
        assert_eq!(
            ws.ordinal,
            idx + 1,
            "session at index {} should have ordinal {}, got {}",
            idx,
            idx + 1,
            ws.ordinal
        );
    }

    // Sessions with known start times must appear in ascending sort order.
    // Sessions with unknown starts use `now` as their display start (for duration
    // ticking) but sort first via MIN_UTC sentinel — so we skip those in the
    // pairwise display-start comparison.
    let known_start_sessions: Vec<_> = sessions
        .iter()
        .filter(|s| {
            // A session whose display `.start` equals `now` has an unknown actual start.
            s.start != now
        })
        .collect();
    for pair in known_start_sessions.windows(2) {
        assert!(
            pair[0].start <= pair[1].start,
            "known-start sessions must be in ascending start-time order: {:?} > {:?}",
            pair[0].start,
            pair[1].start
        );
    }

    // Completed session started before the still-running one (from fixture timestamps)
    let completed_ord = sessions
        .iter()
        .find(|s| s.session_id == "77658ccc-e4f3-4b4d-b734-b75e99c7236c")
        .unwrap()
        .ordinal;
    let running_ord = sessions
        .iter()
        .find(|s| s.session_id == "8bf19191-54bc-4fef-a079-53b33706cf7b")
        .unwrap()
        .ordinal;
    assert!(
        completed_ord < running_ord,
        "completed session (ordinal {completed_ord}) should precede running (ordinal {running_ord})"
    );
}

#[test]
fn val_data_003_running_session_has_positive_duration() {
    // verifies: "VAL-DATA-003: Running WorkerSession duration_secs uses injectable now for determinism"
    let snap = MissionSnapshot::load(&mission_a_dir());
    // now is 2026-06-12T02:00:00Z; worker started at 2026-06-12T00:16:00Z → 6240s
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    let running = sessions
        .iter()
        .find(|s| s.session_id == "8bf19191-54bc-4fef-a079-53b33706cf7b")
        .expect("running session must be present");

    let dur = running
        .duration_secs
        .expect("running session must have duration_secs");
    // start = 00:16:00, now = 02:00:00 → 104 min = 6240s exactly
    assert_eq!(
        dur, 6240,
        "running session duration should be 6240s (now - start), got {dur}"
    );
}

#[test]
fn val_data_003_completed_session_duration_is_end_minus_start() {
    // verifies: "VAL-DATA-003: Completed WorkerSession duration_secs = end timestamp - start timestamp"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let sessions = derive::derive_worker_sessions(&snap, now);

    let completed = sessions
        .iter()
        .find(|s| s.session_id == "77658ccc-e4f3-4b4d-b734-b75e99c7236c")
        .expect("completed session must be present");

    let dur = completed
        .duration_secs
        .expect("completed session must have duration_secs");
    // worker_started at 23:22:15, worker_completed at 00:15:00 next day → 3165s
    assert_eq!(
        dur, 3165,
        "completed session duration should be 3165s (end - start), got {dur}"
    );
}

// ---------------------------------------------------------------------------
// VAL-DATA-004: MissionSummary derived counts, elapsed time, active feature
// ---------------------------------------------------------------------------

#[test]
fn val_data_004_summary_counts_match_feature_statuses() {
    // verifies: "VAL-DATA-004: MissionSummary.completed_features and total_features match feature statuses"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    // fixture has: m1-registry-cli (completed), m1-mission-data-model (in_progress),
    // m1-derived-views (in_progress), m2-tui-shell-dashboard (pending) — 4 total, 1 completed
    assert_eq!(
        summary.total_features,
        snap.features.len(),
        "total_features must equal number of features in snapshot"
    );
    let expected_completed = snap
        .features
        .iter()
        .filter(|f| f.status == "completed")
        .count();
    assert_eq!(
        summary.completed_features, expected_completed,
        "completed_features must count features with status=completed"
    );
    assert!(
        summary.completed_features >= 1,
        "at least 1 feature must be completed (m1-registry-cli)"
    );
    assert!(
        summary.total_features > summary.completed_features,
        "total must exceed completed (some features are in_progress or pending)"
    );
}

#[test]
fn val_data_004_summary_active_feature_is_first_in_progress() {
    // verifies: "VAL-DATA-004: MissionSummary.active_feature is the first in_progress feature"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    let active = summary
        .active_feature
        .expect("active_feature must be Some (fixture has an in_progress feature)");
    assert_eq!(
        active.status, "in_progress",
        "active_feature.status must be in_progress"
    );
    // m1-mission-data-model appears first in features.json with in_progress status
    assert_eq!(
        active.id, "m1-mission-data-model",
        "active_feature.id should be m1-mission-data-model (first in_progress in fixture)"
    );
}

#[test]
fn val_data_004_summary_active_worker_session_id() {
    // verifies: "VAL-DATA-004: MissionSummary.active_worker_session_id matches the active feature's currentWorkerSessionId"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    assert_eq!(
        summary.active_worker_session_id.as_deref(),
        Some("8bf19191-54bc-4fef-a079-53b33706cf7b"),
        "active_worker_session_id must match currentWorkerSessionId of the active feature"
    );
}

#[test]
fn val_data_004_summary_elapsed_secs_is_deterministic() {
    // verifies: "VAL-DATA-004: MissionSummary.elapsed_secs is deterministic when now is injected"
    let snap = MissionSnapshot::load(&mission_a_dir());
    // createdAt = 2026-06-11T23:21:33Z, now = 2026-06-12T02:00:00Z → 9507s
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    let elapsed = summary
        .elapsed_secs
        .expect("elapsed_secs must be Some (createdAt is set in fixture)");
    assert_eq!(
        elapsed, 9507,
        "elapsed_secs should be 9507 (02:00:00 - 23:21:33), got {elapsed}"
    );
}

#[test]
fn val_data_004_summary_state_and_milestone_match_snapshot() {
    // verifies: "VAL-DATA-004: MissionSummary state and milestone fields match the loaded MissionState"
    let snap = MissionSnapshot::load(&mission_a_dir());
    let now = chrono::DateTime::parse_from_rfc3339("2026-06-12T02:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let summary = derive::MissionSummary::from_snapshot(&snap, now);

    assert_eq!(summary.mission_id, snap.state.mission_id);
    assert_eq!(summary.state, snap.state.state);
    assert_eq!(
        summary.milestone.as_deref(),
        snap.state.current_milestone.as_deref(),
        "summary.milestone must mirror state.current_milestone"
    );
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

// ---------------------------------------------------------------------------
// VAL-DATA-003 + VAL-WORK-001: Deterministic ordering regression tests
//
// These tests verify that derive_worker_sessions produces identical session
// ordering across two calls with different `now` values, covering:
//   - Sessions with equal start times (tie-broken by session_id)
//   - Sessions with unknown start (sentinel ordering, not now-dependent)
// ---------------------------------------------------------------------------

/// Build a minimal ProgressEvent for use in regression fixtures.
fn make_event(
    timestamp: &str,
    event_type: &str,
    worker_session_id: &str,
    feature_id: Option<&str>,
    success_state: Option<&str>,
) -> ProgressEvent {
    ProgressEvent {
        timestamp: chrono::DateTime::parse_from_rfc3339(timestamp)
            .unwrap()
            .with_timezone(&Utc),
        event_type: event_type.to_string(),
        worker_session_id: Some(worker_session_id.to_string()),
        feature_id: feature_id.map(|s| s.to_string()),
        success_state: success_state.map(|s| s.to_string()),
        return_to_orchestrator: None,
        message: None,
        milestone: None,
        extra: serde_json::Map::new(),
    }
}

#[test]
fn val_data_003_ordering_stable_across_different_now_values() {
    // verifies: "VAL-DATA-003: derive_worker_sessions ordering is identical for two different now values"
    //
    // Scenario: three sessions
    //   - "aaaa": started at T1 (known start)
    //   - "bbbb": started at T1 (same as "aaaa" — tie-break by session_id)
    //   - "cccc": no worker_started event (unknown start → sentinel ordering)
    //
    // Expected order ascending: cccc (MIN_UTC sentinel) < aaaa (T1, lex first) < bbbb (T1, lex second)
    // With different `now` values the order must remain identical.

    let t1 = "2026-06-01T10:00:00Z";
    let t1_end = "2026-06-01T11:00:00Z";

    let mut snap = MissionSnapshot::default();
    snap.progress_events = vec![
        // "aaaa" starts at T1
        make_event(t1, "worker_started", "aaaa", Some("feat-a"), None),
        make_event(
            t1_end,
            "worker_completed",
            "aaaa",
            Some("feat-a"),
            Some("success"),
        ),
        // "bbbb" also starts at T1 (same timestamp → tied with "aaaa")
        make_event(t1, "worker_started", "bbbb", Some("feat-b"), None),
        make_event(
            t1_end,
            "worker_completed",
            "bbbb",
            Some("feat-b"),
            Some("success"),
        ),
        // "cccc" has NO worker_started event → unknown start (only a completed event)
        make_event(
            t1_end,
            "worker_completed",
            "cccc",
            Some("feat-c"),
            Some("success"),
        ),
    ];

    let now1 = chrono::DateTime::parse_from_rfc3339("2026-06-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let now2 = chrono::DateTime::parse_from_rfc3339("2026-06-01T13:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let sessions1 = derive::derive_worker_sessions(&snap, now1);
    let sessions2 = derive::derive_worker_sessions(&snap, now2);

    assert_eq!(
        sessions1.len(),
        sessions2.len(),
        "both calls must produce the same number of sessions"
    );

    // Extract (ordinal, session_id) pairs for comparison
    let ids1: Vec<(usize, &str)> = sessions1
        .iter()
        .map(|s| (s.ordinal, s.session_id.as_str()))
        .collect();
    let ids2: Vec<(usize, &str)> = sessions2
        .iter()
        .map(|s| (s.ordinal, s.session_id.as_str()))
        .collect();

    assert_eq!(
        ids1, ids2,
        "session ordering (ordinal, session_id) must be identical across different now values;\
         \n  now1 result: {ids1:?}\n  now2 result: {ids2:?}"
    );
}

#[test]
fn val_data_003_unknown_start_uses_sentinel_not_now() {
    // verifies: "VAL-DATA-003: sessions with unknown start sort before known-start sessions (sentinel = MIN_UTC)"
    //
    // A session with no worker_started event must always sort before any session
    // that has a real start timestamp, regardless of what `now` is.

    let t_known = "2026-06-01T10:00:00Z";
    let t_end = "2026-06-01T11:00:00Z";

    let mut snap = MissionSnapshot::default();
    snap.progress_events = vec![
        // "known-session": has a worker_started event
        make_event(
            t_known,
            "worker_started",
            "known-session",
            Some("feat-a"),
            None,
        ),
        make_event(
            t_end,
            "worker_completed",
            "known-session",
            Some("feat-a"),
            Some("success"),
        ),
        // "unknown-session": only completed, no started (sentinel sort key)
        make_event(
            t_end,
            "worker_completed",
            "unknown-session",
            Some("feat-b"),
            Some("success"),
        ),
    ];

    // Use a `now` value that is far in the future — must NOT affect sort order
    let now = chrono::DateTime::parse_from_rfc3339("2030-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let sessions = derive::derive_worker_sessions(&snap, now);
    assert_eq!(sessions.len(), 2, "expected 2 sessions");

    // unknown-session must come first (ordinal 1) because MIN_UTC < any real timestamp
    let unknown = sessions
        .iter()
        .find(|s| s.session_id == "unknown-session")
        .expect("unknown-session must be present");
    let known = sessions
        .iter()
        .find(|s| s.session_id == "known-session")
        .expect("known-session must be present");

    assert_eq!(
        unknown.ordinal, 1,
        "unknown-start session must have ordinal 1 (sorts before known-start); got ordinal {}",
        unknown.ordinal
    );
    assert_eq!(
        known.ordinal, 2,
        "known-start session must have ordinal 2; got ordinal {}",
        known.ordinal
    );
}

#[test]
fn val_data_003_equal_start_tie_broken_by_session_id() {
    // verifies: "VAL-DATA-003: sessions with equal start times are tie-broken deterministically by session_id"
    //
    // Two sessions share the exact same start timestamp. The tie-break must be
    // lexicographic session_id, not HashMap iteration order (which is random).

    let t_start = "2026-06-01T10:00:00Z";
    let t_end = "2026-06-01T11:00:00Z";

    let mut snap = MissionSnapshot::default();
    snap.progress_events = vec![
        make_event(
            t_start,
            "worker_started",
            "zz-last-lex",
            Some("feat-z"),
            None,
        ),
        make_event(
            t_end,
            "worker_completed",
            "zz-last-lex",
            Some("feat-z"),
            Some("success"),
        ),
        make_event(
            t_start,
            "worker_started",
            "aa-first-lex",
            Some("feat-a"),
            None,
        ),
        make_event(
            t_end,
            "worker_completed",
            "aa-first-lex",
            Some("feat-a"),
            Some("success"),
        ),
    ];

    let now = chrono::DateTime::parse_from_rfc3339("2026-06-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    // Run many times to expose any non-determinism (HashMap seed changes each run,
    // but we can also run twice within the same test to check stability).
    let sessions_a = derive::derive_worker_sessions(&snap, now);
    let sessions_b = derive::derive_worker_sessions(&snap, now);

    let order_a: Vec<&str> = sessions_a.iter().map(|s| s.session_id.as_str()).collect();
    let order_b: Vec<&str> = sessions_b.iter().map(|s| s.session_id.as_str()).collect();

    assert_eq!(
        order_a, order_b,
        "repeated calls must yield identical order"
    );

    // The lexicographically smaller id ("aa-first-lex") must come first
    assert_eq!(
        sessions_a[0].session_id, "aa-first-lex",
        "aa-first-lex (lex smaller) must be ordinal 1; order was: {order_a:?}"
    );
    assert_eq!(
        sessions_a[1].session_id, "zz-last-lex",
        "zz-last-lex (lex larger) must be ordinal 2; order was: {order_a:?}"
    );
    assert_eq!(sessions_a[0].ordinal, 1);
    assert_eq!(sessions_a[1].ordinal, 2);
}

#[test]
fn val_work_001_workers_panel_order_stable_across_ticks() {
    // verifies: "VAL-WORK-001: workers panel rows do not reshuffle between poll ticks (now-independent ordering)"
    //
    // This is the primary regression test for the reported HITL bug.
    // Simulates the scenario that caused visible reshuffling:
    //   - Two sessions with identical start times (equal sort key before fix)
    //   - One session with unknown start (used `now` as sort key before fix)
    // Calls derive_worker_sessions with two different `now` values (simulating two
    // consecutive poll ticks) and asserts the resulting sequence of session_ids and
    // ordinals is IDENTICAL.

    let t_start = "2026-06-10T08:00:00Z";
    let t_end = "2026-06-10T09:00:00Z";

    let mut snap = MissionSnapshot::default();
    snap.progress_events = vec![
        // Session "worker-alpha": known start at t_start, still running
        make_event(
            t_start,
            "worker_started",
            "worker-alpha",
            Some("feat-alpha"),
            None,
        ),
        // Session "worker-beta": same known start at t_start (tie with alpha)
        make_event(
            t_start,
            "worker_started",
            "worker-beta",
            Some("feat-beta"),
            None,
        ),
        make_event(
            t_end,
            "worker_completed",
            "worker-beta",
            Some("feat-beta"),
            Some("success"),
        ),
        // Session "worker-gamma": NO worker_started (unknown start → was buggy before fix)
        make_event(
            t_end,
            "worker_completed",
            "worker-gamma",
            Some("feat-gamma"),
            Some("partial"),
        ),
    ];

    // Tick 1: now is T+0
    let now_tick1 = chrono::DateTime::parse_from_rfc3339("2026-06-10T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    // Tick 2: now is T+1s (simulates the next poll tick)
    let now_tick2 = chrono::DateTime::parse_from_rfc3339("2026-06-10T10:00:01Z")
        .unwrap()
        .with_timezone(&Utc);

    let tick1 = derive::derive_worker_sessions(&snap, now_tick1);
    let tick2 = derive::derive_worker_sessions(&snap, now_tick2);

    let seq1: Vec<(usize, &str)> = tick1
        .iter()
        .map(|s| (s.ordinal, s.session_id.as_str()))
        .collect();
    let seq2: Vec<(usize, &str)> = tick2
        .iter()
        .map(|s| (s.ordinal, s.session_id.as_str()))
        .collect();

    assert_eq!(
        seq1, seq2,
        "workers panel row order must be identical on consecutive ticks;\
         \n  tick1: {seq1:?}\n  tick2: {seq2:?}"
    );

    // Also assert expected absolute order:
    // worker-gamma (MIN_UTC sentinel) < worker-alpha (t_start, lex first) < worker-beta (t_start, lex second)
    let ids: Vec<&str> = tick1.iter().map(|s| s.session_id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["worker-gamma", "worker-alpha", "worker-beta"],
        "expected order: gamma (sentinel) → alpha (lex before beta) → beta; got: {ids:?}"
    );

    // Ordinals must be 1-based and match position
    for (i, ws) in tick1.iter().enumerate() {
        assert_eq!(
            ws.ordinal,
            i + 1,
            "ordinal mismatch at index {i}: expected {}, got {}",
            i + 1,
            ws.ordinal
        );
    }

    // The running session (worker-alpha) duration_secs must differ between ticks
    // (proving `now` still influences duration display, just not sort order).
    let alpha_t1 = tick1
        .iter()
        .find(|s| s.session_id == "worker-alpha")
        .unwrap()
        .duration_secs
        .unwrap();
    let alpha_t2 = tick2
        .iter()
        .find(|s| s.session_id == "worker-alpha")
        .unwrap()
        .duration_secs
        .unwrap();
    assert_eq!(
        alpha_t2 - alpha_t1,
        1,
        "running session duration should increase by 1s between ticks (now advances by 1s)"
    );
}
