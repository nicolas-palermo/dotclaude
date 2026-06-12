use crate::app::App;
use crate::data::derive::{derive_worker_sessions, MissionSummary, WorkerStatus};
use crate::ui::theme::{status_color, status_glyph};
use crate::ui::widgets::{format_relative_time, render_footer, Hint};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
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

/// Format elapsed seconds as a human-readable string.
fn format_elapsed(secs: i64) -> String {
    if secs < 0 {
        return "0s".to_owned();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Render the header bar: "∴ Mission Control  <working_dir>" left, "TIME <elapsed>" right.
fn render_header(f: &mut Frame, area: Rect, summary: &MissionSummary, snap_working_dir: &str) {
    let elapsed_str = summary
        .elapsed_secs
        .map(format_elapsed)
        .unwrap_or_else(|| "--".to_owned());

    let left = format!("\u{2234} Mission Control  {snap_working_dir}");
    let right = format!("TIME {elapsed_str}");

    // Pad right to fill terminal width
    let total_width = area.width as usize;
    let left_len = left.len();
    let right_len = right.len();
    let pad = total_width.saturating_sub(left_len + right_len);
    let header_text = format!("{left}{:>pad$}{right}", "", pad = pad);

    let header = Paragraph::new(header_text).style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(header, area);
}

/// Render the status line: colored state badge + progress gauge.
fn render_status_line(f: &mut Frame, area: Rect, summary: &MissionSummary) {
    // Split into badge (left) and gauge (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(14), Constraint::Min(0)])
        .split(area);

    // State badge
    let state = &summary.state;
    let badge_color = status_color(state);
    let badge_text = format!(" {state} ");
    let badge = Paragraph::new(badge_text).style(
        Style::default()
            .fg(badge_color)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(badge, chunks[0]);

    // Progress gauge
    let ratio = if summary.total_features == 0 {
        0.0
    } else {
        summary.completed_features as f64 / summary.total_features as f64
    };
    let label = format!(
        "{}/{} features",
        summary.completed_features, summary.total_features
    );
    // Use an explicit bg on gauge_style so the label swap (filled region inverts fg/bg)
    // gives Black-on-Green. The label Span uses White+Bold so it reads on both the
    // filled (Green bg) and unfilled (Black bg) regions.
    let label_span = Span::styled(
        label,
        Style::default()
            .fg(ratatui::style::Color::White)
            .add_modifier(Modifier::BOLD),
    );
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(
            Style::default()
                .fg(ratatui::style::Color::Green)
                .bg(ratatui::style::Color::Black),
        )
        .ratio(ratio.clamp(0.0, 1.0))
        .label(label_span);
    f.render_widget(gauge, chunks[1]);
}

/// Render the active-feature pane.
fn render_active_feature(f: &mut Frame, area: Rect, summary: &MissionSummary) {
    let block = Block::default()
        .title(" Active Feature ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    match &summary.active_feature {
        None => {
            let text = Paragraph::new("No Active Feature");
            f.render_widget(text, inner);
        }
        Some(feat) => {
            let mut lines: Vec<Line> = Vec::new();

            // ID + skill
            let skill = feat.skill_name.as_deref().unwrap_or("unknown");
            lines.push(Line::from(vec![
                Span::styled(&feat.id, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(skill, Style::default().fg(ratatui::style::Color::Cyan)),
            ]));

            // Milestone
            lines.push(Line::from(vec![
                Span::styled(
                    "Milestone: ",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
                Span::raw(&feat.milestone),
            ]));

            // Description
            lines.push(Line::from(vec![
                Span::styled(
                    "Desc: ",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
                Span::raw(&feat.description),
            ]));

            // Preconditions
            if !feat.preconditions.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Preconditions:",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
                for p in &feat.preconditions {
                    lines.push(Line::from(format!("  • {p}")));
                }
            }

            // Expected behavior
            if !feat.expected_behavior.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Expected:",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
                for b in &feat.expected_behavior {
                    lines.push(Line::from(format!("  • {b}")));
                }
            }

            let para = Paragraph::new(lines);
            f.render_widget(para, inner);
        }
    }
}

/// Render the active-worker pane.
fn render_active_worker(
    f: &mut Frame,
    area: Rect,
    summary: &MissionSummary,
    snap: &crate::data::loader::MissionSnapshot,
    now: chrono::DateTime<Utc>,
) {
    let block = Block::default()
        .title(" Active Worker ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    match &summary.active_worker_session_id {
        None => {
            let text = Paragraph::new("No active worker");
            f.render_widget(text, inner);
        }
        Some(sid) => {
            // Find matching WorkerSession
            let sessions = derive_worker_sessions(snap, now);
            let ws = sessions.iter().find(|w| &w.session_id == sid);
            let mut lines: Vec<Line> = Vec::new();

            // Short session id (first 8 chars)
            let short_id = &sid[..sid.len().min(8)];
            lines.push(Line::from(vec![
                Span::styled(
                    "Session: ",
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
                Span::styled(short_id, Style::default().add_modifier(Modifier::BOLD)),
            ]));

            if let Some(ws) = ws {
                let status_str = ws.status.as_str();
                let status_color = match &ws.status {
                    WorkerStatus::Running => ratatui::style::Color::Cyan,
                    WorkerStatus::Success => ratatui::style::Color::Green,
                    WorkerStatus::Failed => ratatui::style::Color::Red,
                    WorkerStatus::Partial => ratatui::style::Color::Yellow,
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        "Status: ",
                        Style::default().fg(ratatui::style::Color::DarkGray),
                    ),
                    Span::styled(status_str, Style::default().fg(status_color)),
                ]));

                if let Some(dur) = ws.duration_secs {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Running: ",
                            Style::default().fg(ratatui::style::Color::DarkGray),
                        ),
                        Span::raw(format_elapsed(dur)),
                    ]));
                }
            }

            let para = Paragraph::new(lines);
            f.render_widget(para, inner);
        }
    }
}

/// Render the features sidebar.
fn render_features_list(f: &mut Frame, area: Rect, snap: &crate::data::loader::MissionSnapshot) {
    let block = Block::default().title(" Features ").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = snap
        .features
        .iter()
        .map(|feat| {
            let glyph = status_glyph(&feat.status);
            let color = status_color(&feat.status);
            let short_id = feat.id.as_str();
            Line::from(vec![
                Span::styled(glyph, Style::default().fg(color)),
                Span::raw(" "),
                Span::raw(short_id),
            ])
        })
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

/// Render the progress log pane (last N events with relative timestamps).
fn render_progress_log(
    f: &mut Frame,
    area: Rect,
    snap: &crate::data::loader::MissionSnapshot,
    now: chrono::DateTime<Utc>,
) {
    let block = Block::default()
        .title(" Progress Log ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Show last (inner.height) events
    let max_lines = inner.height as usize;
    let events = &snap.progress_events;
    let start = events.len().saturating_sub(max_lines);
    let visible: Vec<_> = events[start..].iter().collect();

    let lines: Vec<Line> = visible
        .iter()
        .map(|ev| {
            let rel = format_relative_time(ev.timestamp, now);
            let event_type = &ev.event_type;
            Line::from(vec![
                Span::styled(
                    format!("{rel:>8} "),
                    Style::default().fg(ratatui::style::Color::DarkGray),
                ),
                Span::raw(event_type.as_str()),
            ])
        })
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

/// Draw the full dashboard screen.
pub fn draw_dashboard(f: &mut Frame, app: &App) {
    let area = f.area();
    let now = Utc::now();

    // Outer vertical layout: header | status | body | footer
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header bar
            Constraint::Length(1), // status line
            Constraint::Min(0),    // body (two columns)
            Constraint::Length(1), // footer hints
        ])
        .split(area);

    let snap = app.snapshot.as_ref();

    if let Some(snap) = snap {
        let summary = MissionSummary::from_snapshot(snap, now);

        // Header
        render_header(f, outer[0], &summary, &snap.state.working_directory);

        // Status line
        render_status_line(f, outer[1], &summary);

        // Body: two columns
        let body_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(outer[2]);

        // Left column: active feature (top) + active worker (bottom)
        let left_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(body_cols[0]);

        render_active_feature(f, left_rows[0], &summary);
        render_active_worker(f, left_rows[1], &summary, snap, now);

        // Right column: features list (top) + progress log (bottom)
        let right_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body_cols[1]);

        render_features_list(f, right_rows[0], snap);
        render_progress_log(f, right_rows[1], snap, now);
    } else {
        // No snapshot: render minimal placeholder header + status
        let placeholder = Paragraph::new("\u{2234} Mission Control")
            .style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(placeholder, outer[0]);

        let no_data = Paragraph::new("No mission data loaded.");
        f.render_widget(no_data, outer[2]);
    }

    // Footer
    render_footer(f, outer[3], DASHBOARD_HINTS);
}
