use crate::app::App;
use crate::ui::widgets::{render_footer, Hint};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

static FEATURES_HINTS: &[Hint] = &[
    Hint {
        key: "↑↓",
        label: "navigate",
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

/// Draw the features panel (placeholder — fully implemented in m3-panels).
pub fn draw_features(f: &mut Frame, _app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new("Features").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    let body = Paragraph::new(
        "Features panel — detail view coming in m3-panels.\n\nPress [Esc] to go back to Dashboard.",
    )
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    render_footer(f, chunks[2], FEATURES_HINTS);
}

/// Draw the feature detail screen (placeholder — fully implemented in m3-panels).
pub fn draw_feature_detail(f: &mut Frame, _app: &App, feature_id: &str) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new(format!("Feature — {feature_id}")).style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    let body = Paragraph::new(
        "Feature detail — coming in m3-panels.\n\nPress [Esc] to go back to Features.",
    )
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(body, chunks[1]);

    static DETAIL_HINTS: &[Hint] = &[
        Hint {
            key: "Esc",
            label: "back",
        },
        Hint {
            key: "q",
            label: "quit",
        },
    ];
    render_footer(f, chunks[2], DETAIL_HINTS);
}
