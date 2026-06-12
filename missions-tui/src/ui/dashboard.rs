use crate::app::App;
use crate::ui::widgets::{render_footer, Hint};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

static DASHBOARD_HINTS: &[Hint] = &[
    Hint {
        key: "F",
        label: "features",
    },
    Hint {
        key: "W",
        label: "workers",
    },
    Hint {
        key: "Esc",
        label: "back",
    },
    Hint {
        key: "q",
        label: "quit",
    },
];

/// Draw the dashboard screen (minimal placeholder — fully implemented in m3).
pub fn draw_dashboard(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let title = app
        .active_mission_dir
        .as_ref()
        .and_then(|d| d.parent())
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "mission".to_owned());

    let header = Paragraph::new(format!("Dashboard — {title}")).style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    // Body placeholder
    let body_text = if let Some(snap) = &app.snapshot {
        let state = &snap.state.state;
        let total = snap.features.len();
        let done = snap
            .features
            .iter()
            .filter(|f| f.status == "completed")
            .count();
        let warnings = snap.warnings.len();
        let warn_str = if warnings > 0 {
            format!("  ({warnings} warnings)")
        } else {
            String::new()
        };
        format!(
            "State: {state}   Features: {done}/{total} completed{warn_str}\n\n[F] view features    [W] view workers"
        )
    } else {
        "No mission data loaded.\n\n[F] view features    [W] view workers".to_owned()
    };

    let body = Paragraph::new(body_text).block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    // Footer
    render_footer(f, chunks[2], DASHBOARD_HINTS);
}
