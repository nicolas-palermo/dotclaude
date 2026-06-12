use crate::app::{filter_features, App, FEATURES_FILTER_COUNT, FILTER_LABELS};
use crate::data::derive::derive_worker_sessions;
use crate::ui::theme::{status_color, status_glyph};
use crate::ui::widgets::{render_footer, Hint};
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// How many feature rows are visible in the body area (minus the tab row + scroll indicator).
/// Rows per visible page = body area height - 1 (tab row) - 1 (scroll indicator).
const TAB_ROW_HEIGHT: u16 = 1;
const SCROLL_INDICATOR_HEIGHT: u16 = 1;

static FEATURES_HINTS: &[Hint] = &[
    Hint {
        key: "T",
        label: "filter",
    },
    Hint {
        key: "↑↓",
        label: "navigate",
    },
    Hint {
        key: "Enter",
        label: "detail",
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

/// Draw the features panel with filter tabs, scrollable list, and scroll indicator.
pub fn draw_features(f: &mut Frame, app: &App) {
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
        "Features",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(header, outer[0]);

    // Body layout: tab row (1) | list area (min) | scroll indicator (1)
    let body_area = outer[1];
    let list_rows = body_area
        .height
        .saturating_sub(TAB_ROW_HEIGHT + SCROLL_INDICATOR_HEIGHT);

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(TAB_ROW_HEIGHT),
            Constraint::Length(list_rows),
            Constraint::Length(SCROLL_INDICATOR_HEIGHT),
        ])
        .split(body_area);

    // Build filter tabs row
    let snap = app.snapshot.as_ref();

    // Counts per filter
    let counts: Vec<usize> = (0..FEATURES_FILTER_COUNT)
        .map(|i| {
            snap.map(|s| filter_features(&s.features, i).len())
                .unwrap_or(0)
        })
        .collect();

    let mut tab_spans: Vec<Span> = Vec::new();
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw("  "));
        }
        let text = format!("{} ({})", label, counts[i]);
        if i == app.features_filter {
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

    // Filtered features
    let filtered: Vec<_> = snap
        .map(|s| filter_features(&s.features, app.features_filter))
        .unwrap_or_default();

    let total = filtered.len();
    let visible = list_rows as usize;

    // Scroll offset: keep list_selected in view
    let scroll_offset = if total == 0 {
        0
    } else {
        let sel = app.list_selected.min(total.saturating_sub(1));
        // Clamp scroll so sel is visible
        let mut offset = app.scroll_offset;
        if sel < offset {
            offset = sel;
        } else if sel >= offset + visible {
            offset = sel.saturating_sub(visible - 1);
        }
        offset
    };

    // Render feature rows
    let window: Vec<_> = filtered
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible)
        .collect();

    let list_area = body_chunks[1];
    let mut row_lines: Vec<Line> = Vec::new();

    for (idx, feature) in &window {
        let glyph = status_glyph(&feature.status);
        let color = status_color(&feature.status);
        let label = format!(
            "{}/{}  {} {}",
            feature.milestone, feature.id, glyph, feature.status
        );
        let is_selected = *idx == app.list_selected.min(total.saturating_sub(1));

        let mut style = Style::default().fg(color);
        if is_selected {
            style = style.add_modifier(Modifier::REVERSED);
        }
        row_lines.push(Line::from(Span::styled(label, style)));
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
    f.render_widget(indicator_para, body_chunks[2]);

    // Footer
    render_footer(f, outer[2], FEATURES_HINTS);
}

/// Draw the feature detail screen showing all fields and worker session status.
pub fn draw_feature_detail(f: &mut Frame, app: &App, feature_id: &str) {
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

    // Header: feature id as title
    let header = Paragraph::new(Line::from(vec![Span::styled(
        format!("Feature: {feature_id}"),
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    f.render_widget(header, outer[0]);

    // Find the feature
    let feature = app
        .snapshot
        .as_ref()
        .and_then(|s| s.features.iter().find(|f| f.id == feature_id));

    let body_area = outer[1];

    let Some(feat) = feature else {
        let not_found = Paragraph::new(format!("Feature '{feature_id}' not found in snapshot."))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(not_found, body_area);
        render_footer(f, outer[2], DETAIL_HINTS);
        return;
    };

    // Build detail lines
    let mut lines: Vec<Line> = Vec::new();

    // Status line
    {
        let glyph = status_glyph(&feat.status);
        let color = status_color(&feat.status);
        lines.push(Line::from(vec![
            Span::raw("Status:    "),
            Span::styled(
                format!("{glyph} {}", feat.status),
                Style::default().fg(color),
            ),
        ]));
    }

    // Milestone
    lines.push(Line::from(format!("Milestone: {}", feat.milestone)));

    // Blank separator
    lines.push(Line::from(""));

    // Description
    lines.push(Line::from(vec![Span::styled(
        "Description",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(format!("  {}", feat.description)));
    lines.push(Line::from(""));

    // Preconditions
    if !feat.preconditions.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Preconditions",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for p in &feat.preconditions {
            lines.push(Line::from(format!("  • {p}")));
        }
        lines.push(Line::from(""));
    }

    // Expected Behavior
    if !feat.expected_behavior.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Expected Behavior",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        for b in &feat.expected_behavior {
            lines.push(Line::from(format!("  • {b}")));
        }
        lines.push(Line::from(""));
    }

    // Worker Sessions
    if !feat.worker_session_ids.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Worker Sessions",
            Style::default().add_modifier(Modifier::BOLD),
        )]));

        // Derive sessions to get status for each session id
        let derived = app
            .snapshot
            .as_ref()
            .map(|s| derive_worker_sessions(s, now))
            .unwrap_or_default();

        for sid in &feat.worker_session_ids {
            // Find derived status for this session id
            let status_str = derived
                .iter()
                .find(|ws| &ws.session_id == sid)
                .map(|ws| ws.status.as_str())
                .unwrap_or("unknown");

            let (glyph, color) = match status_str {
                "Running" => (
                    crate::ui::theme::GLYPH_IN_PROGRESS,
                    crate::ui::theme::STATUS_IN_PROGRESS,
                ),
                "Success" => (
                    crate::ui::theme::GLYPH_COMPLETED,
                    crate::ui::theme::STATUS_COMPLETED,
                ),
                "Partial" => (
                    crate::ui::theme::GLYPH_PARTIAL,
                    crate::ui::theme::STATUS_PARTIAL,
                ),
                "Failed" => (
                    crate::ui::theme::GLYPH_FAILED,
                    crate::ui::theme::STATUS_FAILED,
                ),
                _ => (
                    crate::ui::theme::GLYPH_PENDING,
                    crate::ui::theme::STATUS_PENDING,
                ),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{glyph} {sid}"), Style::default().fg(color)),
                Span::raw(format!("  [{status_str}]")),
            ]));
        }
    }

    let body_para = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
    f.render_widget(body_para, body_area);

    render_footer(f, outer[2], DETAIL_HINTS);
}
