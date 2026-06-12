/// Registry integration tests.
///
/// Config isolation: tests set `MISSIONS_TUI_CONFIG_DIR` to a tempdir so that
/// the `dirs` crate's macOS behaviour (ignoring XDG_CONFIG_HOME) does not
/// affect results.
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Pull in the crate's registry module.
// Integration tests reference the crate by its package name.
use missions_tui::registry;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp dir, set `MISSIONS_TUI_CONFIG_DIR` to it, and return both
/// the guard (keeps tempdir alive) and the resolved config path.
///
/// SAFETY: `std::env::set_var` is not thread-safe in the general case, but
/// Cargo runs each test binary single-threaded by default and each test here
/// works in its own tempdir scope.
fn isolated_config() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().expect("tempdir");
    let config_path = tmp.path().to_path_buf();
    // Override so registry::config_dir() returns this path.
    std::env::set_var("MISSIONS_TUI_CONFIG_DIR", &config_path);
    (tmp, config_path)
}

/// Assert that the repos.json file at `config_dir` contains exactly the given
/// list of canonical path strings (order-independent).
fn assert_registry_paths(config_dir: &Path, expected: &[&str]) {
    let reg = registry::load(config_dir);
    let actual: Vec<String> = reg
        .repos
        .iter()
        .map(|r| r.path.to_string_lossy().to_string())
        .collect();
    let mut expected_sorted: Vec<&str> = expected.to_vec();
    expected_sorted.sort();
    let mut actual_sorted = actual.clone();
    actual_sorted.sort();
    assert_eq!(
        actual_sorted,
        expected_sorted
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        "registry contents mismatch; actual={actual:?} expected={expected:?}"
    );
}

// ---------------------------------------------------------------------------
// VAL-REG-001: repo add canonicalizes, persists, and deduplicates
// ---------------------------------------------------------------------------

#[test]
fn val_reg_001_add_persists_and_dedupes() {
    let (_tmp, config_dir) = isolated_config();
    let repo_tmp = TempDir::new().expect("repo tempdir");
    let repo_path = repo_tmp.path();

    // Add a real directory.
    registry::add(&config_dir, repo_path).expect("first add should succeed");

    // Registry should contain exactly one entry (compare canonicalized).
    let canonical_repo = repo_path.canonicalize().unwrap();
    assert_registry_paths(&config_dir, &[&canonical_repo.to_string_lossy()]);

    // Add the same path a second time — should not duplicate.
    registry::add(&config_dir, repo_path).expect("second add (dedup) should succeed");
    assert_registry_paths(&config_dir, &[&canonical_repo.to_string_lossy()]);

    // Verify the file was created.
    let file = config_dir.join("repos.json");
    assert!(file.exists(), "repos.json should exist after add");
}

#[test]
fn val_reg_001_add_canonicalizes_path() {
    let (_tmp, config_dir) = isolated_config();
    let repo_tmp = TempDir::new().expect("repo tempdir");
    let repo_path = repo_tmp.path();

    // Build a non-canonical path using `..` if possible.
    // e.g. /tmp/foo/../foo → /tmp/foo
    let sub = repo_path.join("sub");
    fs::create_dir_all(&sub).unwrap();
    let non_canonical = sub.join("..").join("sub");

    registry::add(&config_dir, &non_canonical).expect("add with non-canonical path");
    let reg = registry::load(&config_dir);
    assert_eq!(reg.repos.len(), 1);
    // The stored path should equal the canonicalized sub dir.
    let canonical = sub.canonicalize().unwrap();
    assert_eq!(reg.repos[0].path, canonical);
}

// ---------------------------------------------------------------------------
// VAL-REG-002: repo list shows active mission; repo remove deletes entry
// ---------------------------------------------------------------------------

#[test]
fn val_reg_002_list_no_active_mission() {
    let (_tmp, config_dir) = isolated_config();
    let repo_tmp = TempDir::new().expect("repo tempdir");
    let repo_path = repo_tmp.path();

    registry::add(&config_dir, repo_path).expect("add");

    let reg = registry::load(&config_dir);
    assert_eq!(reg.repos.len(), 1);

    // No active-mission.txt → discover_active_mission returns None.
    let mission_info = registry::discover_active_mission(repo_path);
    assert!(
        mission_info.is_none(),
        "should be None when active-mission.txt is absent"
    );
}

#[test]
fn val_reg_002_list_with_active_mission() {
    let (_tmp, config_dir) = isolated_config();
    let repo_tmp = TempDir::new().expect("repo tempdir");
    let repo_path = repo_tmp.path();

    // Create a fake active-mission.txt inside the repo.
    let missions_dir = repo_path.join(".claude").join("missions");
    fs::create_dir_all(&missions_dir).unwrap();
    let mission_id = "test-mission-uuid-001";
    fs::write(missions_dir.join("active-mission.txt"), mission_id).unwrap();

    // Create a minimal state.json for that mission.
    let mission_dir = missions_dir.join(mission_id);
    fs::create_dir_all(&mission_dir).unwrap();
    fs::write(
        mission_dir.join("state.json"),
        r#"{"missionId":"test-mission-uuid-001","state":"running","workingDirectory":"/tmp"}"#,
    )
    .unwrap();

    registry::add(&config_dir, repo_path).expect("add");

    let (found_id, found_dir) =
        registry::discover_active_mission(repo_path).expect("should find active mission");
    assert_eq!(found_id, mission_id);
    // Canonicalize both sides to handle macOS /var → /private/var symlink.
    let found_dir_canonical = found_dir
        .canonicalize()
        .unwrap_or_else(|_| found_dir.clone());
    let mission_dir_canonical = mission_dir.canonicalize().unwrap_or(mission_dir);
    assert_eq!(found_dir_canonical, mission_dir_canonical);

    let state = registry::read_mission_state(&found_dir);
    assert_eq!(state.as_deref(), Some("running"));
}

#[test]
fn val_reg_002_remove_deletes_entry() {
    let (_tmp, config_dir) = isolated_config();
    let repo1 = TempDir::new().expect("repo1");
    let repo2 = TempDir::new().expect("repo2");

    registry::add(&config_dir, repo1.path()).expect("add repo1");
    registry::add(&config_dir, repo2.path()).expect("add repo2");
    assert_eq!(registry::load(&config_dir).repos.len(), 2);

    registry::remove(&config_dir, repo1.path()).expect("remove repo1");
    let reg = registry::load(&config_dir);
    assert_eq!(reg.repos.len(), 1);
    // repo1 should be gone; repo2 should remain.
    let canonical2 = repo2.path().canonicalize().unwrap();
    assert_eq!(reg.repos[0].path, canonical2);
}

// ---------------------------------------------------------------------------
// VAL-REG-003: repo add on nonexistent path fails cleanly, registry untouched
// ---------------------------------------------------------------------------

#[test]
fn val_reg_003_nonexistent_path_fails_with_error() {
    let (_tmp, config_dir) = isolated_config();
    let nonexistent = std::path::PathBuf::from("/this/path/does/not/exist/ever");

    // Pre-populate with a real repo so we can verify the registry is untouched.
    let real_repo = TempDir::new().expect("real repo");
    registry::add(&config_dir, real_repo.path()).expect("pre-add real repo");

    // Capture registry state before the failing add.
    let before = registry::load(&config_dir);
    let before_paths: Vec<_> = before.repos.iter().map(|r| r.path.clone()).collect();

    // Attempt to add a nonexistent path — must return an error.
    let result = registry::add(&config_dir, &nonexistent);
    assert!(result.is_err(), "add of nonexistent path must return Err");

    // Error message should mention the path.
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("does not exist") || err_msg.contains("/this/path"),
        "error should mention the bad path, got: {err_msg}"
    );

    // Registry must be unchanged.
    let after = registry::load(&config_dir);
    let after_paths: Vec<_> = after.repos.iter().map(|r| r.path.clone()).collect();
    assert_eq!(
        before_paths, after_paths,
        "registry must be untouched after failed add"
    );
}

#[test]
fn val_reg_003_nonexistent_path_non_zero_conceptual_exit() {
    // This test verifies the Err return (which main.rs maps to a non-zero exit code
    // via `?` propagation from main() -> anyhow::Result<()>).
    let (_tmp, config_dir) = isolated_config();
    let bad = std::path::PathBuf::from("/no/such/path/xyz");
    let result = registry::add(&config_dir, &bad);
    assert!(
        result.is_err(),
        "add on nonexistent path must return Err (maps to non-zero exit)"
    );
}
