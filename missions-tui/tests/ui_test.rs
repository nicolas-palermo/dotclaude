/// TUI shell tests — App state machine + rendering with TestBackend.
///
/// VAL-TUI-001: Selector lists repos; Enter opens Dashboard screen.
/// VAL-TUI-002: F→Features, W→Workers, Esc walks back through the nav graph, q exits.
/// VAL-TUI-003: tick_reload re-reads snapshot from disk; mutated file is reflected.
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
