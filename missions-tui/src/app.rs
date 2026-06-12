use crate::data::derive::{derive_worker_sessions, WorkerStatus};
use crate::data::loader::MissionSnapshot;
use crate::data::model::Feature;
use crate::registry::{self, RepoEntry};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

/// All screens in the navigation graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Selector,
    Dashboard,
    Features,
    FeatureDetail(String),
    Workers,
}

/// An entry in the selector list: a repo path plus discovered mission info.
#[derive(Debug, Clone)]
pub struct SelectorEntry {
    pub repo_path: PathBuf,
    /// Active mission id discovered from `active-mission.txt`, if present.
    pub mission_id: Option<String>,
    /// Mission dir path (for loading snapshot), if present.
    pub mission_dir: Option<PathBuf>,
    /// Mission state string, if discoverable.
    pub state: Option<String>,
}

impl SelectorEntry {
    pub fn from_repo(entry: &RepoEntry) -> Self {
        match registry::discover_active_mission(&entry.path) {
            Some((id, dir)) => {
                let state = registry::read_mission_state(&dir);
                SelectorEntry {
                    repo_path: entry.path.clone(),
                    mission_id: Some(id),
                    mission_dir: Some(dir),
                    state,
                }
            }
            None => SelectorEntry {
                repo_path: entry.path.clone(),
                mission_id: None,
                mission_dir: None,
                state: None,
            },
        }
    }
}

/// Filter index for the features panel.
/// 0=All, 1=Pending, 2=InProgress, 3=Completed, 4=Cancelled
pub const FEATURES_FILTER_COUNT: usize = 5;

/// Application state. Free of all rendering and IO concerns.
pub struct App {
    /// Current screen.
    pub screen: Screen,
    /// Repo entries for the selector screen.
    pub selector_entries: Vec<SelectorEntry>,
    /// Currently highlighted index in the selector list.
    pub selector_selected: usize,
    /// The active snapshot for the mission currently being viewed.
    pub snapshot: Option<MissionSnapshot>,
    /// Mission dir for the currently selected entry (used for tick reloads).
    pub active_mission_dir: Option<PathBuf>,
    /// If true, the event loop should exit.
    pub should_quit: bool,
    /// Scroll/selection offset in the features/workers list.
    pub list_selected: usize,
    /// Scroll offset for long views.
    pub scroll_offset: usize,
    /// Active filter tab in the features panel (0=All, 1=Pending, 2=InProgress, 3=Completed, 4=Cancelled).
    pub features_filter: usize,
    /// Active filter tab in the workers panel (0=All, 1=Active, 2=Completed, 3=Failed).
    pub workers_filter: usize,
}

impl App {
    /// Create a new App, loading the repo registry from `config_dir`.
    pub fn new(config_dir: &std::path::Path) -> Self {
        let reg = registry::load(config_dir);
        let selector_entries: Vec<SelectorEntry> =
            reg.repos.iter().map(SelectorEntry::from_repo).collect();

        App {
            screen: Screen::Selector,
            selector_entries,
            selector_selected: 0,
            snapshot: None,
            active_mission_dir: None,
            should_quit: false,
            list_selected: 0,
            scroll_offset: 0,
            features_filter: 0,
            workers_filter: 0,
        }
    }

    /// Create an App with a pre-built selector entry list (for tests).
    pub fn with_entries(entries: Vec<SelectorEntry>) -> Self {
        App {
            screen: Screen::Selector,
            selector_entries: entries,
            selector_selected: 0,
            snapshot: None,
            active_mission_dir: None,
            should_quit: false,
            list_selected: 0,
            scroll_offset: 0,
            features_filter: 0,
            workers_filter: 0,
        }
    }

    /// The selector entry currently highlighted.
    fn selected_entry(&self) -> Option<&SelectorEntry> {
        self.selector_entries.get(self.selector_selected)
    }

    /// Open the dashboard for the currently selected selector entry.
    fn open_dashboard(&mut self) {
        if let Some(entry) = self.selected_entry() {
            if let Some(dir) = entry.mission_dir.clone() {
                let snap = MissionSnapshot::load(&dir);
                self.snapshot = Some(snap);
                self.active_mission_dir = Some(dir);
            } else {
                self.snapshot = None;
                self.active_mission_dir = None;
            }
        }
        self.list_selected = 0;
        self.scroll_offset = 0;
        self.screen = Screen::Dashboard;
    }

    /// Reload the snapshot from the active mission dir (called on every ~1s tick).
    /// No-op if no mission dir is set.
    pub fn tick_reload(&mut self) {
        if let Some(dir) = &self.active_mission_dir.clone() {
            self.snapshot = Some(MissionSnapshot::load(dir));
        }
    }

    /// Handle a key event and update state accordingly. Returns `true` if handled.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Ctrl-C always quits.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return true;
        }

        match key.code {
            // q — quit from any screen
            KeyCode::Char('q') => {
                self.should_quit = true;
                true
            }

            // Esc — walk back
            KeyCode::Esc => {
                match &self.screen {
                    Screen::Selector => {
                        self.should_quit = true;
                    }
                    Screen::Dashboard => {
                        self.screen = Screen::Selector;
                        self.active_mission_dir = None;
                        self.snapshot = None;
                    }
                    Screen::Features | Screen::Workers => {
                        self.screen = Screen::Dashboard;
                    }
                    Screen::FeatureDetail(_) => {
                        self.screen = Screen::Features;
                    }
                }
                true
            }

            // F/f — go to features panel (from Dashboard, Workers, or same)
            KeyCode::Char('F') | KeyCode::Char('f') => {
                match &self.screen {
                    Screen::Dashboard
                    | Screen::Features
                    | Screen::FeatureDetail(_)
                    | Screen::Workers => {
                        self.list_selected = 0;
                        self.scroll_offset = 0;
                        self.screen = Screen::Features;
                    }
                    Screen::Selector => {}
                }
                true
            }

            // W/w — go to workers panel (from Dashboard, Features, or same)
            KeyCode::Char('W') | KeyCode::Char('w') => {
                match &self.screen {
                    Screen::Dashboard
                    | Screen::Features
                    | Screen::FeatureDetail(_)
                    | Screen::Workers => {
                        self.list_selected = 0;
                        self.scroll_offset = 0;
                        self.screen = Screen::Workers;
                    }
                    Screen::Selector => {}
                }
                true
            }

            // Enter — in selector, open dashboard; in features list, open detail
            KeyCode::Enter => {
                match self.screen.clone() {
                    Screen::Selector => {
                        self.open_dashboard();
                    }
                    Screen::Features => {
                        // Open feature detail for the selected feature in the filtered list.
                        if let Some(snap) = &self.snapshot {
                            let filtered = filter_features(&snap.features, self.features_filter);
                            if let Some(feature) = filtered.get(self.list_selected) {
                                let id = feature.id.clone();
                                self.screen = Screen::FeatureDetail(id);
                            }
                        }
                    }
                    _ => {}
                }
                true
            }

            // ↑ — move selection up in lists / selector
            KeyCode::Up => {
                match self.screen {
                    Screen::Selector => {
                        if self.selector_selected > 0 {
                            self.selector_selected -= 1;
                        }
                    }
                    Screen::Features | Screen::Workers => {
                        if self.list_selected > 0 {
                            self.list_selected -= 1;
                        }
                    }
                    _ => {}
                }
                true
            }

            // ↓ — move selection down
            KeyCode::Down => {
                match self.screen {
                    Screen::Selector => {
                        let max = self.selector_entries.len().saturating_sub(1);
                        if self.selector_selected < max {
                            self.selector_selected += 1;
                        }
                    }
                    Screen::Features => {
                        if let Some(snap) = &self.snapshot {
                            let filtered = filter_features(&snap.features, self.features_filter);
                            let max = filtered.len().saturating_sub(1);
                            if self.list_selected < max {
                                self.list_selected += 1;
                            }
                        }
                    }
                    Screen::Workers => {
                        // Worker count: derived externally; let it grow freely (capped in render)
                        self.list_selected = self.list_selected.saturating_add(1);
                    }
                    _ => {}
                }
                true
            }

            // g — jump to top
            KeyCode::Char('g') => {
                self.list_selected = 0;
                self.scroll_offset = 0;
                true
            }

            // G — jump to bottom (features and workers lists)
            KeyCode::Char('G') => {
                if let Some(snap) = &self.snapshot {
                    match self.screen {
                        Screen::Features => {
                            let filtered = filter_features(&snap.features, self.features_filter);
                            self.list_selected = filtered.len().saturating_sub(1);
                        }
                        Screen::Workers => {
                            let now = Utc::now();
                            let all = derive_worker_sessions(snap, now);
                            let filtered = filter_workers(&all, self.workers_filter);
                            self.list_selected = filtered.len().saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                true
            }

            // Tab — cycle the active filter tab in the features or workers panel
            KeyCode::Tab => {
                match self.screen {
                    Screen::Features => {
                        self.features_filter = (self.features_filter + 1) % FEATURES_FILTER_COUNT;
                        self.list_selected = 0;
                        self.scroll_offset = 0;
                    }
                    Screen::Workers => {
                        self.workers_filter = (self.workers_filter + 1) % WORKERS_FILTER_COUNT;
                        self.list_selected = 0;
                        self.scroll_offset = 0;
                    }
                    _ => {}
                }
                true
            }

            _ => false,
        }
    }
}

/// Filter tab count for the workers panel: All / Active / Completed / Failed.
pub const WORKERS_FILTER_COUNT: usize = 4;

/// Filter tab labels for the features panel.
pub const FILTER_LABELS: [&str; FEATURES_FILTER_COUNT] =
    ["All", "Pending", "In Progress", "Completed", "Cancelled"];

/// Filter tab status strings (None = All).
pub const FILTER_STATUSES: [Option<&str>; FEATURES_FILTER_COUNT] = [
    None,
    Some("pending"),
    Some("in_progress"),
    Some("completed"),
    Some("cancelled"),
];

/// Return the subset of features matching the given filter index.
/// filter 0 = All (no filtering).
pub fn filter_features(features: &[Feature], filter: usize) -> Vec<&Feature> {
    match FILTER_STATUSES.get(filter).copied().flatten() {
        None => features.iter().collect(),
        Some(status) => features.iter().filter(|f| f.status == status).collect(),
    }
}

/// Worker filter tab labels: All / Active / Completed / Failed.
pub const WORKERS_FILTER_LABELS: [&str; WORKERS_FILTER_COUNT] =
    ["All", "Active", "Completed", "Failed"];

/// Filter worker sessions by the given tab index.
/// 0=All, 1=Active (Running), 2=Completed (Success), 3=Failed (Failed+Partial)
pub fn filter_workers(
    sessions: &[crate::data::derive::WorkerSession],
    filter: usize,
) -> Vec<&crate::data::derive::WorkerSession> {
    match filter {
        1 => sessions
            .iter()
            .filter(|s| s.status == WorkerStatus::Running)
            .collect(),
        2 => sessions
            .iter()
            .filter(|s| s.status == WorkerStatus::Success)
            .collect(),
        3 => sessions
            .iter()
            .filter(|s| s.status == WorkerStatus::Failed || s.status == WorkerStatus::Partial)
            .collect(),
        _ => sessions.iter().collect(), // 0 = All
    }
}
