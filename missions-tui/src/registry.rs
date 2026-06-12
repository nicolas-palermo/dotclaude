use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A single registered repo entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoEntry {
    pub path: PathBuf,
}

/// The persisted registry: a JSON array of repo entries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    pub repos: Vec<RepoEntry>,
}

/// Determine the config directory for missions-tui.
///
/// Priority:
/// 1. `MISSIONS_TUI_CONFIG_DIR` env var (used by tests and overrides)
/// 2. `dirs::config_dir()` / `missions-tui`
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(override_dir) = std::env::var("MISSIONS_TUI_CONFIG_DIR") {
        return Ok(PathBuf::from(override_dir));
    }
    let base = dirs::config_dir().context("could not determine config directory")?;
    Ok(base.join("missions-tui"))
}

fn registry_path(config: &Path) -> PathBuf {
    config.join("repos.json")
}

/// Load the registry from disk. Returns an empty registry if the file is absent.
/// Logs a warning and returns empty on corrupt JSON (never panics).
pub fn load(config_dir: &Path) -> Registry {
    let path = registry_path(config_dir);
    match fs::read_to_string(&path) {
        Err(_) => Registry::default(),
        Ok(text) => match serde_json::from_str::<Registry>(&text) {
            Ok(reg) => reg,
            Err(e) => {
                eprintln!("warning: repos.json is corrupt, ignoring ({e})");
                Registry::default()
            }
        },
    }
}

/// Persist the registry to disk, creating parent dirs as needed.
pub fn save(config_dir: &Path, registry: &Registry) -> Result<()> {
    fs::create_dir_all(config_dir)
        .with_context(|| format!("could not create config dir: {}", config_dir.display()))?;
    let path = registry_path(config_dir);
    let json = serde_json::to_string_pretty(registry).context("serialization error")?;
    fs::write(&path, json)
        .with_context(|| format!("could not write registry: {}", path.display()))?;
    Ok(())
}

/// Add a path to the registry.
///
/// - Canonicalizes the path (requires it to exist).
/// - Deduplicates: silently ignores already-registered paths.
/// - Returns an error (without touching the registry) if path does not exist.
pub fn add(config_dir: &Path, raw_path: &Path) -> Result<()> {
    if !raw_path.exists() {
        bail!("path does not exist: {}", raw_path.display());
    }
    let canonical = raw_path
        .canonicalize()
        .with_context(|| format!("could not canonicalize path: {}", raw_path.display()))?;

    let mut registry = load(config_dir);
    if registry.repos.iter().any(|r| r.path == canonical) {
        // Already present — deduplication, nothing to do.
        return Ok(());
    }
    registry.repos.push(RepoEntry { path: canonical });
    save(config_dir, &registry)
}

/// Remove a path from the registry. Canonicalization is attempted; if it fails,
/// we fall back to removing by the literal path.
pub fn remove(config_dir: &Path, raw_path: &Path) -> Result<()> {
    let target = raw_path
        .canonicalize()
        .unwrap_or_else(|_| raw_path.to_path_buf());

    let mut registry = load(config_dir);
    let before = registry.repos.len();
    registry.repos.retain(|r| r.path != target);
    if registry.repos.len() == before {
        eprintln!(
            "warning: path not found in registry: {}",
            raw_path.display()
        );
    }
    save(config_dir, &registry)
}

/// Discover the active mission for a repo: read `<repo>/.claude/missions/active-mission.txt`.
/// Returns `(mission_id, mission_dir)` or `None` if absent / unreadable.
pub fn discover_active_mission(repo: &Path) -> Option<(String, PathBuf)> {
    let txt = repo.join(".claude/missions/active-mission.txt");
    let mission_id = fs::read_to_string(&txt).ok()?.trim().to_string();
    if mission_id.is_empty() {
        return None;
    }
    let mission_dir = repo.join(".claude/missions").join(&mission_id);
    Some((mission_id, mission_dir))
}

/// Read the mission state string from a mission directory's `state.json`.
/// Returns `None` if absent or corrupt.
pub fn read_mission_state(mission_dir: &Path) -> Option<String> {
    let text = fs::read_to_string(mission_dir.join("state.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    v.get("state")?.as_str().map(|s| s.to_string())
}
