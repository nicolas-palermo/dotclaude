use crate::app::{filter_workers, App, WORKERS_FILTER_COUNT, WORKERS_FILTER_LABELS};
use crate::data::derive::{derive_worker_sessions, WorkerStatus};
use crate::ui::theme::{STATUS_COMPLETED, STATUS_FAILED, STATUS_IN_PROGRESS, STATUS_PARTIAL};
use crate::ui::widgets::{render_footer, Hint};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const TAB_ROW_HEIGHT: u16 = 1;
const HEADER_ROW_HEIGHT: u16 = 1;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

static WORKERS_HINTS: &[Hint] = &[
    Hint {
        key: "Tab",
        label: "filter",
    },
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

/// Format duration seconds as "Xm Ys" or "Xh Ym".
fn format_duration(secs: i64) -> String {
    if secs < 0 {
        return "0s".to_owned();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m {}s", m, s)
    }
}

/// Color for a WorkerStatus.
fn worker_status_color(status: &WorkerStatus) -> ratatui::style::Color {
    match status {
        WorkerStatus::Running => STATUS_IN_PROGRESS,
        WorkerStatus::Success => STATUS_COMPLETED,
        WorkerStatus::Failed => STATUS_FAILED,
        WorkerStatus::Partial => STATUS_PARTIAL,
    }
}

/// Draw the workers panel with filter tabs, column headers, scrollable list, and scroll indicator.
pub fn draw_workers(f: &mut Frame, app: &App) {
    let now = Utc::now();
    let area = f.area();

    // Outer layout: header (1) | body (min) | footer (1)
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![Span::styled(
        "Workers",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(header, outer[0]);

    // Body layout: tab row (1) | col header row (1) | list area (min) | scroll indicator (1)
    let body_area = outer[1];
    let list_rows = body_area
        .height
        .saturating_sub(TAB_ROW_HEIGHT + HEADER_ROW_HEIGHT + SCROLL_INDICATOR_HEIGHT);

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TAB_ROW_HEIGHT),
            Constraint::Length(HEADER_ROW_HEIGHT),
            Constraint::Length(list_rows),
            Constraint::Length(SCROLL_INDICATOR_HEIGHT),
        ])
        .split(body_area);

    // Derive all worker sessions (ascending start order)
    let snap = app.snapshot.as_ref();
    let all_sessions: Vec<_> = snap
        .map(|s| derive_worker_sessions(s, now))
        .unwrap_or_default();

    // Build filter tab counts
    let counts: Vec<usize> = (0..WORKERS_FILTER_COUNT)
        .map(|i| filter_workers(&all_sessions, i).len())
        .collect();

    // Filter tabs row
    let mut tab_spans: Vec<Span> = Vec::new();
    for (i, label) in WORKERS_FILTER_LABELS.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw("  "));
        }
        let text = format!("{} ({})", label, counts[i]);
        if i == app.workers_filter {
            tab_spans.push(Span::styled(
                text,
                Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else {
            tab_spans.push(Span::raw(text));
        }
    }
    let tabs = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tabs, body_chunks[0]);

    // Column header row
    let col_header = Paragraph::new(Line::from(vec![Span::styled(
        format!(
            "{:<3}  {:<8}  {:<5}  {:<8}  {:<9}  {}",
            "#", "Session", "Start", "Duration", "Status", "Feature"
        ),
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(col_header, body_chunks[1]);

    // Filtered sessions — newest-first (reverse of ascending order)
    let mut filtered: Vec<_> = filter_workers(&all_sessions, app.workers_filter);
    filtered.reverse();

    let total = filtered.len();
    let visible = list_rows as usize;

    // Clamp list_selected to valid range
    let sel = if total == 0 {
        0
    } else {
        app.list_selected.min(total.saturating_sub(1))
    };

    // Scroll offset: keep sel in view
    let scroll_offset = if total == 0 {
        0
    } else {
        let mut offset = app.scroll_offset;
        if sel < offset {
            offset = sel;
        } else if sel >= offset + visible {
            offset = sel.saturating_sub(visible.saturating_sub(1));
        }
        offset
    };

    // Render worker rows
    let window: Vec<_> = filtered
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible)
        .collect();

    let list_area = body_chunks[2];
    let mut row_lines: Vec<Line> = Vec::new();

    for (idx, ws) in &window {
        let session_short = if ws.session_id.len() >= 8 {
            &ws.session_id[..8]
        } else {
            &ws.session_id
        };
        let start_str = ws.start.format("%H:%M").to_string();
        let duration_str = ws
            .duration_secs
            .map(format_duration)
            .unwrap_or_else(|| "—".to_owned());
        let status_str = ws.status.as_str();
        let feature_str = ws.feature_id.as_deref().unwrap_or("—");

        let color = worker_status_color(&ws.status);
        let is_selected = *idx == sel;

        // Build spans: prefix plain, status colored
        let prefix = format!(
            "{:<3}  {:<8}  {:<5}  {:<8}  ",
            ws.ordinal, session_short, start_str, duration_str
        );
        let suffix = format!("  {}", feature_str);

        let mut base_style = Style::default();
        if is_selected {
            base_style = base_style.add_modifier(Modifier::REVERSED);
        }
        let status_style = Style::default().fg(color).add_modifier(if is_selected {
            Modifier::REVERSED
        } else {
            Modifier::empty()
        });

        row_lines.push(Line::from(vec![
            Span::styled(prefix, base_style),
            Span::styled(format!("{:<9}", status_str), status_style),
            Span::styled(suffix, base_style),
        ]));
    }

    // Fill remaining rows with empty lines
    while row_lines.len() < visible {
        row_lines.push(Line::from(""));
    }

    let list_para = Paragraph::new(row_lines);
    f.render_widget(list_para, list_area);

    // Scroll indicator
    let indicator = if total == 0 {
        "showing 0 of 0".to_owned()
    } else {
        let first = scroll_offset + 1;
        let last = (scroll_offset + visible).min(total);
        format!("showing {first}–{last} of {total}")
    };
    let indicator_para = Paragraph::new(Line::from(Span::raw(indicator)));
    f.render_widget(indicator_para, body_chunks[3]);

    // Footer
    render_footer(f, outer[2], WORKERS_HINTS);
}
