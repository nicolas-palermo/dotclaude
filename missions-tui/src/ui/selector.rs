use crate::app::App;
use crate::ui::theme;
use crate::ui::widgets::{render_footer, Hint};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

static SELECTOR_HINTS: &[Hint] = &[
    Hint {
        key: "↑↓",
        label: "navigate",
    },
    Hint {
        key: "Enter",
        label: "open",
    },
    Hint {
        key: "q",
        label: "quit",
    },
];

/// Draw the repo selector screen.
pub fn draw_selector(f: &mut Frame, app: &App) {
    let area = f.area();

    // Split into header (1), list (fill), footer (1).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let header = Paragraph::new("missions-tui — select a repository").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(header, chunks[0]);

    // Build list items
    let items: Vec<ListItem> = app
        .selector_entries
        .iter()
        .map(|entry| {
            let path_str = entry
                .repo_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| entry.repo_path.to_string_lossy().into_owned());

            match (&entry.mission_id, &entry.state) {
                (Some(id), Some(state)) => {
                    let glyph = theme::status_glyph(state);
                    let color = theme::status_color(state);
                    let short_id = if id.len() > 8 { &id[..8] } else { id.as_str() };
                    let line = Line::from(vec![
                        Span::styled(format!("{glyph} "), Style::default().fg(color)),
                        Span::raw(format!("{path_str}  ")),
                        Span::styled(
                            format!("[{short_id}…]"),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(format!("  {state}"), Style::default().fg(color)),
                    ]);
                    ListItem::new(line)
                }
                (Some(id), None) => {
                    let short_id = if id.len() > 8 { &id[..8] } else { id.as_str() };
                    let line = Line::from(vec![
                        Span::raw(format!("○ {path_str}  ")),
                        Span::styled(
                            format!("[{short_id}…]"),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]);
                    ListItem::new(line)
                }
                _ => {
                    let line = Line::from(Span::styled(
                        format!("  {path_str}  (no mission)"),
                        Style::default().fg(Color::DarkGray),
                    ));
                    ListItem::new(line)
                }
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !app.selector_entries.is_empty() {
        list_state.select(Some(app.selector_selected));
    }

    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Footer
    render_footer(f, chunks[2], SELECTOR_HINTS);
}
