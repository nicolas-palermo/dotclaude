use crate::app::App;
use crate::ui::widgets::{render_footer, Hint};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

static WORKERS_HINTS: &[Hint] = &[
    Hint {
        key: "↑↓",
        label: "navigate",
    },
    Hint {
        key: "F",
        label: "features",
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

/// Draw the workers panel (placeholder — fully implemented in m3-panels).
pub fn draw_workers(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new("Workers").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    let worker_count = app
        .snapshot
        .as_ref()
        .map(|s| s.progress_events.len())
        .unwrap_or(0);

    let body = Paragraph::new(format!(
        "Workers panel — detail view coming in m3-panels.\n\n{worker_count} progress events in snapshot.\n\nPress [Esc] to go back to Dashboard."
    ))
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    render_footer(f, chunks[2], WORKERS_HINTS);
}
