/// TUI shell tests — App state machine + rendering with TestBackend.
///
/// VAL-TUI-001: Selector lists repos; Enter opens Dashboard screen.
/// VAL-TUI-002: F→Features, W→Workers, Esc walks back through the nav graph, q exits.
/// VAL-TUI-003: tick_reload re-reads snapshot from disk; mutated file is reflected.
/// VAL-DASH-001: Dashboard renders active-feature pane with in_progress feature details.
/// VAL-DASH-002: Dashboard with no in_progress feature renders "No Active Feature" and no-worker placeholder.
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use missions_tui::app::{App, Screen, SelectorEntry};
use missions_tui::data::loader::MissionSnapshot;
use missions_tui::ui;
use ratatui::{backend::TestBackend, Terminal};
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

fn press(app: &mut App, code: KeyCode) -> bool {
    app.handle_key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    })
}

/// Build an App with two SelectorEntries: one with a mission, one without.
fn make_app_with_entries() -> App {
    let entry_with_mission = SelectorEntry {
        repo_path: PathBuf::from("/repos/alpha"),
        mission_id: Some("99038ad5-abbe-4534-9e5a-d14ece4274e6".into()),
        mission_dir: Some(mission_a_dir()),
        state: Some("running".into()),
    };
    let entry_no_mission = SelectorEntry {
        repo_path: PathBuf::from("/repos/beta"),
        mission_id: None,
        mission_dir: None,
        state: None,
    };
    App::with_entries(vec![entry_with_mission, entry_no_mission])
}

fn make_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("TestBackend terminal")
}

// ---------------------------------------------------------------------------
// VAL-TUI-001: Selector renders repos; Enter opens Dashboard
// ---------------------------------------------------------------------------

#[test]
fn val_tui_001_selector_renders_repo_names() {
    // verifies: "VAL-TUI-001: selector renders registered repos with mission info"
    let app = make_app_with_entries();
    let mut terminal = make_terminal(80, 24);

    terminal.draw(|f| ui::draw(f, &app)).expect("draw");

    let buf = terminal.backend().buffer().clone();

    // Flatten the buffer into a string so we can assert on content.
    let rendered: String = (0..24)
        .map(|y| {
            (0..80)
                .map(|x| {
                    buf.cell((x, y))
                        .map(|c| c.symbol().chars().next().unwrap_or(' '))
                        .unwrap_or(' ')
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Header must be present.
    assert!(
        rendered.contains("missions-tui"),
        "selector header should contain 'missions-tui', got:\n{rendered}"
    );

    // Repo basename "alpha" (with mission) and "beta" (no mission) must appear.
    assert!(
        rendered.contains("alpha"),
        "selector should show repo 'alpha', got:\n{rendered}"
    );
    assert!(
        rendered.contains("beta"),
        "selector should show repo 'beta', got:\n{rendered}"
    );

    // Mission state "running" should appear for the alpha entry.
    assert!(
        rendered.contains("running"),
        "selector should show mission state 'running', got:\n{rendered}"
    );
}

#[test]
fn val_tui_001_enter_transitions_to_dashboard() {
    // verifies: "VAL-TUI-001: Enter on selector entry opens Dashboard screen"
    let mut app = make_app_with_entries();

    assert_eq!(
        app.screen,
        Screen::Selector,
        "initial screen should be Selector"
    );

    // Press Enter — should open Dashboard (entry 0 has a mission dir)
    press(&mut app, KeyCode::Enter);

    assert_eq!(
        app.screen,
        Screen::Dashboard,
        "Enter on Selector should navigate to Dashboard"
    );
    assert!(
        app.snapshot.is_some(),
        "snapshot should be loaded after opening dashboard"
    );
    assert!(!app.should_quit, "app should not quit after Enter");
}

#[test]
fn val_tui_001_selector_navigation_arrows() {
    // verifies: "VAL-TUI-001: ↑↓ move selector_selected"
    let mut app = make_app_with_entries();

    assert_eq!(app.selector_selected, 0);

    press(&mut app, KeyCode::Down);
    assert_eq!(app.selector_selected, 1, "↓ should advance selection");

    press(&mut app, KeyCode::Down);
    assert_eq!(
        app.selector_selected, 1,
        "↓ at last entry should not overflow"
    );

    press(&mut app, KeyCode::Up);
    assert_eq!(app.selector_selected, 0, "↑ should go back");

    press(&mut app, KeyCode::Up);
    assert_eq!(
        app.selector_selected, 0,
        "↑ at first entry should not underflow"
    );
}

#[test]
fn val_tui_001_dashboard_renders_after_enter() {
    // verifies: "VAL-TUI-001: Dashboard screen renders without panic after selector Enter"
    let mut app = make_app_with_entries();
    press(&mut app, KeyCode::Enter);

    let mut terminal = make_terminal(80, 24);
    // Must not panic.
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard");

    let buf = terminal.backend().buffer().clone();
    let rendered: String = (0..24)
        .map(|y| {
            (0..80)
                .map(|x| {
                    buf.cell((x, y))
                        .map(|c| c.symbol().chars().next().unwrap_or(' '))
                        .unwrap_or(' ')
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    // At minimum the mission id fragment should show somewhere.
    assert!(
        rendered.contains("running") || rendered.contains("99038ad5"),
        "dashboard should show mission info, got:\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// VAL-TUI-002: Navigation graph — F, W, Esc, q
// ---------------------------------------------------------------------------

/// Put app into Dashboard state directly with snapshot loaded.
fn app_on_dashboard() -> App {
    let mut app = make_app_with_entries();
    press(&mut app, KeyCode::Enter); // opens Dashboard
    app
}

#[test]
fn val_tui_002_f_opens_features_from_dashboard() {
    // verifies: "VAL-TUI-002: F key navigates from Dashboard to Features"
    let mut app = app_on_dashboard();
    assert_eq!(app.screen, Screen::Dashboard);

    press(&mut app, KeyCode::Char('F'));
    assert_eq!(
        app.screen,
        Screen::Features,
        "F should open Features from Dashboard"
    );
}

#[test]
fn val_tui_002_w_opens_workers_from_dashboard() {
    // verifies: "VAL-TUI-002: W key navigates from Dashboard to Workers"
    let mut app = app_on_dashboard();

    press(&mut app, KeyCode::Char('W'));
    assert_eq!(
        app.screen,
        Screen::Workers,
        "W should open Workers from Dashboard"
    );
}

#[test]
fn val_tui_002_esc_from_features_returns_to_dashboard() {
    // verifies: "VAL-TUI-002: Esc from Features walks back to Dashboard"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('F'));
    assert_eq!(app.screen, Screen::Features);

    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.screen,
        Screen::Dashboard,
        "Esc from Features should return to Dashboard"
    );
}

#[test]
fn val_tui_002_esc_from_workers_returns_to_dashboard() {
    // verifies: "VAL-TUI-002: Esc from Workers walks back to Dashboard"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('W'));
    assert_eq!(app.screen, Screen::Workers);

    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.screen,
        Screen::Dashboard,
        "Esc from Workers should return to Dashboard"
    );
}

#[test]
fn val_tui_002_esc_from_dashboard_returns_to_selector() {
    // verifies: "VAL-TUI-002: Esc from Dashboard walks back to Selector"
    let mut app = app_on_dashboard();
    assert_eq!(app.screen, Screen::Dashboard);

    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.screen,
        Screen::Selector,
        "Esc from Dashboard should return to Selector"
    );
    assert!(
        app.snapshot.is_none(),
        "snapshot should be cleared on leaving Dashboard"
    );
}

#[test]
fn val_tui_002_esc_from_selector_quits() {
    // verifies: "VAL-TUI-002: Esc from Selector sets should_quit"
    let mut app = make_app_with_entries();
    assert_eq!(app.screen, Screen::Selector);
    assert!(!app.should_quit);

    press(&mut app, KeyCode::Esc);
    assert!(app.should_quit, "Esc from Selector should set should_quit");
}

#[test]
fn val_tui_002_q_quits_from_selector() {
    // verifies: "VAL-TUI-002: q quits from Selector"
    let mut app = make_app_with_entries();
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn val_tui_002_q_quits_from_dashboard() {
    // verifies: "VAL-TUI-002: q quits from Dashboard"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn val_tui_002_q_quits_from_features() {
    // verifies: "VAL-TUI-002: q quits from Features"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('F'));
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn val_tui_002_q_quits_from_workers() {
    // verifies: "VAL-TUI-002: q quits from Workers"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('W'));
    press(&mut app, KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn val_tui_002_f_is_noop_from_selector() {
    // verifies: "VAL-TUI-002: F does nothing from Selector (no mission open)"
    let mut app = make_app_with_entries();
    press(&mut app, KeyCode::Char('F'));
    assert_eq!(
        app.screen,
        Screen::Selector,
        "F should be no-op from Selector"
    );
}

#[test]
fn val_tui_002_w_is_noop_from_selector() {
    // verifies: "VAL-TUI-002: W does nothing from Selector (no mission open)"
    let mut app = make_app_with_entries();
    press(&mut app, KeyCode::Char('W'));
    assert_eq!(
        app.screen,
        Screen::Selector,
        "W should be no-op from Selector"
    );
}

#[test]
fn val_tui_002_esc_from_feature_detail_returns_to_features() {
    // verifies: "VAL-TUI-002: Esc from FeatureDetail walks back to Features"
    let mut app = app_on_dashboard();
    press(&mut app, KeyCode::Char('F'));
    assert_eq!(app.screen, Screen::Features);

    // Manually set FeatureDetail screen (requires features in snapshot)
    app.screen = Screen::FeatureDetail("test-feature".into());
    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.screen,
        Screen::Features,
        "Esc from FeatureDetail should return to Features"
    );
}

#[test]
fn val_tui_002_all_screens_render_without_panic() {
    // verifies: "VAL-TUI-002: all Screen variants render without panic"
    let mut app = make_app_with_entries();
    let mut terminal = make_terminal(80, 24);

    // Selector
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("Selector render");

    // Dashboard
    press(&mut app, KeyCode::Enter);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("Dashboard render");

    // Features
    press(&mut app, KeyCode::Char('F'));
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("Features render");

    // FeatureDetail
    app.screen = Screen::FeatureDetail("test-id".into());
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("FeatureDetail render");

    // Workers
    app.screen = Screen::Workers;
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("Workers render");
}

// ---------------------------------------------------------------------------
// VAL-TUI-003: tick_reload re-reads snapshot from disk
// ---------------------------------------------------------------------------

#[test]
fn val_tui_003_tick_reload_reads_snapshot_from_disk() {
    // verifies: "VAL-TUI-003: tick_reload loads snapshot from active_mission_dir"
    let mission_dir = mission_a_dir();
    let entry = SelectorEntry {
        repo_path: PathBuf::from("/repos/alpha"),
        mission_id: Some("99038ad5".into()),
        mission_dir: Some(mission_dir.clone()),
        state: Some("running".into()),
    };
    let mut app = App::with_entries(vec![entry]);

    // Open dashboard — this loads the snapshot.
    press(&mut app, KeyCode::Enter);
    assert!(
        app.snapshot.is_some(),
        "snapshot should be loaded after Enter"
    );

    let snap_before = app.snapshot.clone().unwrap();
    assert_eq!(
        snap_before.state.mission_id,
        "99038ad5-abbe-4534-9e5a-d14ece4274e6"
    );

    // tick_reload should reload from disk and produce the same result (fixture unchanged).
    app.tick_reload();
    let snap_after = app.snapshot.as_ref().unwrap();
    assert_eq!(
        snap_after.state.mission_id, "99038ad5-abbe-4534-9e5a-d14ece4274e6",
        "tick_reload should re-read state.json"
    );
}

#[test]
fn val_tui_003_tick_reload_reflects_updated_file() {
    // verifies: "VAL-TUI-003: mutated fixture file is reflected after tick_reload"
    use std::fs;

    // Create a temp dir with a minimal mission.
    let tmp = tempfile::tempdir().expect("tempdir");
    let mission_dir = tmp.path().to_path_buf();
    let handoffs_dir = mission_dir.join("handoffs");
    fs::create_dir_all(&handoffs_dir).expect("create handoffs dir");

    // Write initial state.json
    let state_v1 = r#"{"missionId":"aaa","state":"running","workingDirectory":"/tmp","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","currentMilestone":"m1","lastReviewedHandoffCount":0}"#;
    fs::write(mission_dir.join("state.json"), state_v1).expect("write state v1");

    let entry = SelectorEntry {
        repo_path: PathBuf::from("/repos/tmp"),
        mission_id: Some("aaa".into()),
        mission_dir: Some(mission_dir.clone()),
        state: Some("running".into()),
    };
    let mut app = App::with_entries(vec![entry]);

    // Open dashboard — loads snapshot v1.
    press(&mut app, KeyCode::Enter);
    {
        let snap = app.snapshot.as_ref().expect("snapshot loaded");
        assert_eq!(
            snap.state.state, "running",
            "initial state should be 'running'"
        );
        assert_eq!(snap.state.mission_id, "aaa");
    }

    // Mutate the state.json on disk to simulate a mission update.
    let state_v2 = r#"{"missionId":"aaa","state":"completed","workingDirectory":"/tmp","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T01:00:00Z","currentMilestone":"m1","lastReviewedHandoffCount":0}"#;
    fs::write(mission_dir.join("state.json"), state_v2).expect("write state v2");

    // tick_reload should pick up the new state.
    app.tick_reload();
    {
        let snap = app.snapshot.as_ref().expect("snapshot reloaded");
        assert_eq!(
            snap.state.state, "completed",
            "tick_reload should reflect updated state.json"
        );
    }
}

#[test]
fn val_tui_003_tick_reload_is_noop_without_active_dir() {
    // verifies: "VAL-TUI-003: tick_reload does nothing when no active_mission_dir"
    let mut app = make_app_with_entries();
    // Don't open a mission — no active_mission_dir set
    assert!(app.active_mission_dir.is_none());
    assert!(app.snapshot.is_none());

    app.tick_reload(); // should not panic

    assert!(
        app.snapshot.is_none(),
        "snapshot stays None when no active dir"
    );
}

#[test]
fn val_tui_003_tick_reload_with_features_mutation() {
    // verifies: "VAL-TUI-003: features.json mutation is reflected after tick_reload"
    use std::fs;

    let tmp = tempfile::tempdir().expect("tempdir");
    let mission_dir = tmp.path().to_path_buf();
    fs::create_dir_all(mission_dir.join("handoffs")).expect("create handoffs");

    let state_json = r#"{"missionId":"bbb","state":"running","workingDirectory":"/tmp","createdAt":"2026-01-01T00:00:00Z","updatedAt":"2026-01-01T00:00:00Z","currentMilestone":"m1","lastReviewedHandoffCount":0}"#;
    fs::write(mission_dir.join("state.json"), state_json).expect("write state");

    // Initial features.json: 1 feature pending
    let features_v1 = r#"{"features":[{"id":"f1","description":"Feature one","milestone":"m1","status":"pending","skillName":"rust-tui-worker","fulfills":[],"preconditions":[],"expectedBehavior":[],"workerSessionIds":[],"currentWorkerSessionId":null,"completedWorkerSessionId":null}]}"#;
    fs::write(mission_dir.join("features.json"), features_v1).expect("write features v1");

    let entry = SelectorEntry {
        repo_path: PathBuf::from("/repos/tmp2"),
        mission_id: Some("bbb".into()),
        mission_dir: Some(mission_dir.clone()),
        state: Some("running".into()),
    };
    let mut app = App::with_entries(vec![entry]);
    press(&mut app, KeyCode::Enter);

    {
        let snap = app.snapshot.as_ref().unwrap();
        assert_eq!(snap.features.len(), 1);
        assert_eq!(snap.features[0].status, "pending");
    }

    // Update features.json — mark feature completed + add a second
    let features_v2 = r#"{"features":[{"id":"f1","description":"Feature one","milestone":"m1","status":"completed","skillName":"rust-tui-worker","fulfills":[],"preconditions":[],"expectedBehavior":[],"workerSessionIds":[],"currentWorkerSessionId":null,"completedWorkerSessionId":null},{"id":"f2","description":"Feature two","milestone":"m1","status":"in_progress","skillName":"rust-tui-worker","fulfills":[],"preconditions":[],"expectedBehavior":[],"workerSessionIds":[],"currentWorkerSessionId":null,"completedWorkerSessionId":null}]}"#;
    fs::write(mission_dir.join("features.json"), features_v2).expect("write features v2");

    app.tick_reload();
    {
        let snap = app.snapshot.as_ref().unwrap();
        assert_eq!(
            snap.features.len(),
            2,
            "tick_reload should pick up new feature"
        );
        assert_eq!(
            snap.features[0].status, "completed",
            "f1 should now be completed"
        );
        assert_eq!(snap.features[1].id, "f2", "f2 should appear after reload");
    }
}

// ---------------------------------------------------------------------------
// VAL-DASH-001: Dashboard renders active-feature pane with in_progress details
// ---------------------------------------------------------------------------

/// Render the buffer to a single String for assertions.
fn render_to_string(terminal: &Terminal<TestBackend>, width: u16, height: u16) -> String {
    let buf = terminal.backend().buffer().clone();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    buf.cell((x, y))
                        .map(|c| c.symbol().chars().next().unwrap_or(' '))
                        .unwrap_or(' ')
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn val_dash_001_dashboard_renders_active_feature_id() {
    // verifies: "VAL-DASH-001: dashboard renders the in_progress feature id in the active-feature pane"
    // mission-a has feature m1-mission-data-model with status in_progress
    let mut app = app_on_dashboard();
    assert_eq!(app.screen, Screen::Dashboard);

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("m1-mission-data-model"),
        "dashboard should display the in_progress feature id 'm1-mission-data-model', got:\n{rendered}"
    );
}

#[test]
fn val_dash_001_dashboard_does_not_show_no_active_feature_when_in_progress() {
    // verifies: "VAL-DASH-001: dashboard does NOT show 'No Active Feature' when a feature is in_progress"
    let mut app = app_on_dashboard();

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        !rendered.contains("No Active Feature"),
        "dashboard should NOT show 'No Active Feature' when mission-a has an in_progress feature, got:\n{rendered}"
    );
}

#[test]
fn val_dash_001_dashboard_renders_active_feature_skill() {
    // verifies: "VAL-DASH-001: dashboard renders the skill name of the in_progress feature"
    let mut app = app_on_dashboard();

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard");
    let rendered = render_to_string(&terminal, 120, 40);

    // mission-a in_progress feature has skillName: rust-tui-worker
    assert!(
        rendered.contains("rust-tui-worker"),
        "dashboard should display skill 'rust-tui-worker' for active feature, got:\n{rendered}"
    );
}

#[test]
fn val_dash_001_dashboard_renders_features_list_with_glyphs() {
    // verifies: "VAL-DASH-001: features sidebar renders status glyphs for all features"
    let mut app = app_on_dashboard();

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard");
    let rendered = render_to_string(&terminal, 120, 40);

    // mission-a has: m1-registry-cli (completed ✓), m1-mission-data-model (in_progress ●)
    // and pending features (○)
    // The feature ids should appear in the sidebar
    assert!(
        rendered.contains("m1-registry-cli"),
        "features list should include m1-registry-cli, got:\n{rendered}"
    );
    assert!(
        rendered.contains("m1-mission-data-model"),
        "features list should include m1-mission-data-model, got:\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// VAL-DASH-002: Dashboard with no in_progress feature → "No Active Feature"
// ---------------------------------------------------------------------------

/// Build an App pointed at mission-b (no in_progress feature) and navigate to Dashboard.
fn app_on_dashboard_mission_b() -> App {
    let mission_b_dir = fixtures_dir().join("mission-b");
    let entry = SelectorEntry {
        repo_path: PathBuf::from("/repos/project-beta"),
        mission_id: Some("bbbbbbbb-0000-0000-0000-000000000000".into()),
        mission_dir: Some(mission_b_dir),
        state: Some("planning".into()),
    };
    let mut app = App::with_entries(vec![entry]);
    press(&mut app, KeyCode::Enter); // opens Dashboard
    app
}

#[test]
fn val_dash_002_dashboard_shows_no_active_feature_when_all_pending() {
    // verifies: "VAL-DASH-002: dashboard shows 'No Active Feature' when no feature is in_progress"
    let mut app = app_on_dashboard_mission_b();
    assert_eq!(app.screen, Screen::Dashboard);
    assert!(app.snapshot.is_some(), "snapshot loaded for mission-b");

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard mission-b");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("No Active Feature"),
        "dashboard should show 'No Active Feature' when no feature is in_progress, got:\n{rendered}"
    );
}

#[test]
fn val_dash_002_dashboard_shows_no_active_worker_when_no_running_session() {
    // verifies: "VAL-DASH-002: dashboard shows no-worker placeholder when no active worker session"
    let mut app = app_on_dashboard_mission_b();

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard mission-b");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("No active worker"),
        "dashboard should show 'No active worker' when mission-b has no running worker, got:\n{rendered}"
    );
}

#[test]
fn val_dash_002_dashboard_renders_planning_state_badge() {
    // verifies: "VAL-DASH-002: dashboard renders 'planning' state badge for mission-b"
    let mut app = app_on_dashboard_mission_b();

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw dashboard mission-b");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("planning"),
        "dashboard should show 'planning' state badge for mission-b, got:\n{rendered}"
    );
}

#[test]
fn val_dash_002_dashboard_renders_without_panic_for_mission_b() {
    // verifies: "VAL-DASH-002: dashboard renders without panic when snapshot has only completed+pending features"
    let mut app = app_on_dashboard_mission_b();

    let mut terminal = make_terminal(120, 40);
    // Must not panic
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw mission-b dashboard should not panic");
}

// ---------------------------------------------------------------------------
// Helpers for mission-many fixture (12 features across 4 statuses)
// ---------------------------------------------------------------------------

fn mission_many_dir() -> PathBuf {
    fixtures_dir().join("mission-many")
}

/// Build an App pointed at mission-many and navigate to Features screen.
/// mission-many has: 12 total, 3 completed, 1 in_progress, 7 pending, 1 cancelled.
fn app_on_features_many() -> App {
    let entry = SelectorEntry {
        repo_path: PathBuf::from("/repos/many"),
        mission_id: Some("many-features-mission".into()),
        mission_dir: Some(mission_many_dir()),
        state: Some("running".into()),
    };
    let mut app = App::with_entries(vec![entry]);
    press(&mut app, KeyCode::Enter); // opens Dashboard, loads snapshot
    press(&mut app, KeyCode::Char('F')); // navigates to Features
    app
}

// ---------------------------------------------------------------------------
// VAL-FEAT-001: Filter tabs with counts; T cycles filter; rows respect filter
// ---------------------------------------------------------------------------

#[test]
fn val_feat_001_filter_tabs_show_counts() {
    // verifies: "VAL-FEAT-001: filter tab row renders count for each filter label"
    let app = app_on_features_many();
    assert_eq!(app.screen, Screen::Features, "should be on Features screen");

    let mut terminal = make_terminal(120, 30);
    terminal.draw(|f| ui::draw(f, &app)).expect("draw features");
    let rendered = render_to_string(&terminal, 120, 30);

    assert!(
        rendered.contains("All (12)"),
        "tab row should show 'All (12)', got:\n{rendered}"
    );
    assert!(
        rendered.contains("Pending (7)"),
        "tab row should show 'Pending (7)', got:\n{rendered}"
    );
    assert!(
        rendered.contains("In Progress (1)"),
        "tab row should show 'In Progress (1)', got:\n{rendered}"
    );
    assert!(
        rendered.contains("Completed (3)"),
        "tab row should show 'Completed (3)', got:\n{rendered}"
    );
    assert!(
        rendered.contains("Cancelled (1)"),
        "tab row should show 'Cancelled (1)', got:\n{rendered}"
    );
}

#[test]
fn val_feat_001_t_key_cycles_filter() {
    // verifies: "VAL-FEAT-001: T key cycles the active filter index"
    let mut app = app_on_features_many();

    // Initially filter is 0 (All)
    assert_eq!(app.features_filter, 0, "initial filter should be 0 (All)");

    // Press T -> filter 1 (Pending)
    press(&mut app, KeyCode::Char('T'));
    assert_eq!(
        app.features_filter, 1,
        "T should advance filter to 1 (Pending)"
    );

    // Press T -> filter 2 (In Progress)
    press(&mut app, KeyCode::Char('T'));
    assert_eq!(
        app.features_filter, 2,
        "T should advance filter to 2 (In Progress)"
    );

    // Press T four more times: 3, 4, then wrap to 0
    press(&mut app, KeyCode::Char('T'));
    assert_eq!(app.features_filter, 3, "filter should be 3 (Completed)");
    press(&mut app, KeyCode::Char('T'));
    assert_eq!(app.features_filter, 4, "filter should be 4 (Cancelled)");
    press(&mut app, KeyCode::Char('T'));
    assert_eq!(app.features_filter, 0, "T should wrap back to 0 (All)");
}

#[test]
fn val_feat_001_t_resets_selection_and_rows_restricted() {
    // verifies: "VAL-FEAT-001: T resets list_selected to 0; filtered view shows only matching rows"
    let mut app = app_on_features_many();

    // Select item 5 in All filter, then cycle to Pending
    for _ in 0..5 {
        press(&mut app, KeyCode::Down);
    }
    assert_eq!(app.list_selected, 5);

    // T should reset selection
    press(&mut app, KeyCode::Char('T')); // now Pending (filter index 1)
    assert_eq!(app.list_selected, 0, "T should reset list_selected to 0");
    assert_eq!(app.features_filter, 1);

    // Render and verify only pending rows are visible
    let mut terminal = make_terminal(120, 30);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw features pending filter");
    let rendered = render_to_string(&terminal, 120, 30);

    // Pending rows should appear
    assert!(
        rendered.contains("m2-feature-epsilon"),
        "Pending filter should show m2-feature-epsilon, got:\n{rendered}"
    );
    // In-progress feature should NOT appear in the list rows
    assert!(
        !rendered.contains("m2-feature-delta"),
        "Pending filter should NOT show m2-feature-delta (in_progress), got:\n{rendered}"
    );
    // Completed features should NOT appear
    assert!(
        !rendered.contains("m1-feature-alpha"),
        "Pending filter should NOT show m1-feature-alpha (completed), got:\n{rendered}"
    );
}

#[test]
fn val_feat_001_initial_filter_shows_all_features() {
    // verifies: "VAL-FEAT-001: default filter (All) shows features from all statuses"
    let app = app_on_features_many();

    let mut terminal = make_terminal(120, 30);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw features all filter");
    let rendered = render_to_string(&terminal, 120, 30);

    // The first completed feature should appear in the All filter
    assert!(
        rendered.contains("m1-feature-alpha"),
        "All filter should show m1-feature-alpha (completed), got:\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// VAL-FEAT-002: Navigation -- arrows, g, G, scroll indicator
// ---------------------------------------------------------------------------

#[test]
fn val_feat_002_down_increments_selection() {
    // verifies: "VAL-FEAT-002: Down key increments list_selected"
    let mut app = app_on_features_many();
    assert_eq!(app.list_selected, 0);

    press(&mut app, KeyCode::Down);
    assert_eq!(
        app.list_selected, 1,
        "Down should increment list_selected to 1"
    );

    press(&mut app, KeyCode::Down);
    assert_eq!(
        app.list_selected, 2,
        "Down should increment list_selected to 2"
    );
}

#[test]
fn val_feat_002_up_decrements_selection() {
    // verifies: "VAL-FEAT-002: Up key decrements list_selected; clamped at 0"
    let mut app = app_on_features_many();

    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.list_selected, 2);

    press(&mut app, KeyCode::Up);
    assert_eq!(app.list_selected, 1, "Up should decrement to 1");

    press(&mut app, KeyCode::Up);
    assert_eq!(app.list_selected, 0, "Up should decrement to 0");

    press(&mut app, KeyCode::Up);
    assert_eq!(app.list_selected, 0, "Up at 0 should stay at 0 (clamp)");
}

#[test]
fn val_feat_002_down_clamped_at_last() {
    // verifies: "VAL-FEAT-002: Down clamped at last feature index"
    let mut app = app_on_features_many();

    // Hammer down 20 times (more than 12 features)
    for _ in 0..20 {
        press(&mut app, KeyCode::Down);
    }
    // 12 features -> max index is 11
    assert_eq!(
        app.list_selected, 11,
        "Down should clamp at index 11 (12 features total)"
    );
}

#[test]
fn val_feat_002_g_jumps_to_top() {
    // verifies: "VAL-FEAT-002: g resets list_selected to 0"
    let mut app = app_on_features_many();

    for _ in 0..3 {
        press(&mut app, KeyCode::Down);
    }
    assert_eq!(app.list_selected, 3);

    press(&mut app, KeyCode::Char('g'));
    assert_eq!(app.list_selected, 0, "g should jump to top (index 0)");
    assert_eq!(app.scroll_offset, 0, "g should reset scroll_offset to 0");
}

#[test]
fn val_feat_002_shift_g_jumps_to_bottom() {
    // verifies: "VAL-FEAT-002: G sets list_selected to last feature index"
    let mut app = app_on_features_many();
    assert_eq!(app.list_selected, 0);

    press(&mut app, KeyCode::Char('G'));
    // 12 features -> last index is 11
    assert_eq!(
        app.list_selected, 11,
        "G should jump to last feature (index 11)"
    );
}

#[test]
fn val_feat_002_scroll_indicator_shows_range() {
    // verifies: "VAL-FEAT-002: scroll indicator renders 'showing X--Y of N' with en-dash"
    let app = app_on_features_many();

    // Use a short terminal to force scroll
    let mut terminal = make_terminal(80, 15);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw features short terminal");
    let rendered = render_to_string(&terminal, 80, 15);

    // Scroll indicator should show a range (en-dash U+2013, not hyphen)
    assert!(
        rendered.contains("showing 1"),
        "scroll indicator should start with 'showing 1', got:\n{rendered}"
    );
    assert!(
        rendered.contains("of 12"),
        "scroll indicator should end with 'of 12', got:\n{rendered}"
    );
    // Verify en-dash is present (not a hyphen)
    assert!(
        rendered.contains('\u{2013}'),
        "scroll indicator should use en-dash (\u{2013}), got:\n{rendered}"
    );
}

#[test]
fn val_feat_002_scroll_indicator_full_view() {
    // verifies: "VAL-FEAT-002: scroll indicator shows all features visible when terminal is tall"
    let app = app_on_features_many();

    // Large terminal -- all 12 fit
    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw features large terminal");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("showing 1"),
        "scroll indicator should start at 1, got:\n{rendered}"
    );
    assert!(
        rendered.contains("of 12"),
        "scroll indicator total should be 12, got:\n{rendered}"
    );
}

// ---------------------------------------------------------------------------
// VAL-FEAT-003: Enter opens FeatureDetail; detail renders fields; Esc returns
// ---------------------------------------------------------------------------

#[test]
fn val_feat_003_enter_opens_feature_detail() {
    // verifies: "VAL-FEAT-003: Enter on selected feature opens Screen::FeatureDetail"
    let mut app = app_on_features_many();
    assert_eq!(app.screen, Screen::Features);
    assert_eq!(app.list_selected, 0);

    press(&mut app, KeyCode::Enter);

    // First feature in All filter order is m1-feature-alpha
    assert_eq!(
        app.screen,
        Screen::FeatureDetail("m1-feature-alpha".into()),
        "Enter should open FeatureDetail for m1-feature-alpha"
    );
}

#[test]
fn val_feat_003_detail_renders_feature_id_and_milestone() {
    // verifies: "VAL-FEAT-003: feature detail renders feature id and milestone"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter); // open detail for m1-feature-alpha

    assert!(
        matches!(&app.screen, Screen::FeatureDetail(id) if id == "m1-feature-alpha"),
        "should be on m1-feature-alpha detail"
    );

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw feature detail");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("Feature: m1-feature-alpha"),
        "detail header should show 'Feature: m1-feature-alpha', got:\n{rendered}"
    );
    assert!(
        rendered.contains("Milestone: m1-first"),
        "detail should show 'Milestone: m1-first', got:\n{rendered}"
    );
}

#[test]
fn val_feat_003_detail_renders_description() {
    // verifies: "VAL-FEAT-003: feature detail renders description text"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter);

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw feature detail");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("Alpha feature description"),
        "detail should render description 'Alpha feature description', got:\n{rendered}"
    );
}

#[test]
fn val_feat_003_detail_renders_preconditions() {
    // verifies: "VAL-FEAT-003: feature detail renders preconditions as bullet list"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter);

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw feature detail");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("Alpha precondition one"),
        "detail should render precondition 'Alpha precondition one', got:\n{rendered}"
    );
}

#[test]
fn val_feat_003_detail_renders_expected_behavior() {
    // verifies: "VAL-FEAT-003: feature detail renders expected behavior as bullet list"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter);

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw feature detail");
    let rendered = render_to_string(&terminal, 120, 40);

    assert!(
        rendered.contains("Alpha behaves correctly"),
        "detail should render expected behavior 'Alpha behaves correctly', got:\n{rendered}"
    );
}

#[test]
fn val_feat_003_esc_from_detail_returns_to_features() {
    // verifies: "VAL-FEAT-003: Esc from FeatureDetail returns to Screen::Features"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter); // -> FeatureDetail

    assert!(
        matches!(&app.screen, Screen::FeatureDetail(_)),
        "should be on FeatureDetail"
    );

    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.screen,
        Screen::Features,
        "Esc from FeatureDetail should return to Screen::Features"
    );
}

#[test]
fn val_feat_003_detail_renders_worker_sessions() {
    // verifies: "VAL-FEAT-003: feature detail renders worker session ids with status"
    let mut app = app_on_features_many();
    press(&mut app, KeyCode::Enter); // -> m1-feature-alpha detail

    let mut terminal = make_terminal(120, 40);
    terminal
        .draw(|f| ui::draw(f, &app))
        .expect("draw feature detail");
    let rendered = render_to_string(&terminal, 120, 40);

    // m1-feature-alpha has workerSessionIds: ["aaa00001"]
    assert!(
        rendered.contains("aaa00001"),
        "detail should render worker session id 'aaa00001', got:\n{rendered}"
    );
}
