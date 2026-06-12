use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Format a timestamp as a relative string relative to `now`.
///
/// - < 60 min → "3m ago"
/// - < 24 h   → "1h ago"
/// - older    → "06/10" (month/day)
///
/// The `now` parameter is injectable for deterministic tests.
pub fn format_relative_time(ts: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = now.signed_duration_since(ts);
    let secs = delta.num_seconds();
    if secs < 0 {
        // Future timestamp — show as "0m ago"
        return "0m ago".to_owned();
    }
    let mins = delta.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }
    // Older: show as MM/DD
    let day = ts.format("%m/%d").to_string();
    day
}

/// A key-hint pair to display in the footer bar.
pub struct Hint {
    pub key: &'static str,
    pub label: &'static str,
}

/// Render a footer hint bar at the bottom of `area`.
/// Hints are rendered as `[key] label  [key] label  ...`
pub fn render_footer(f: &mut Frame, area: Rect, hints: &[Hint]) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, hint) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!("[{}]", hint.key),
            Style::default().fg(Color::Cyan),
        ));
        spans.push(Span::raw(format!(" {}", hint.label)));
    }
    let line = Line::from(spans);
    let para = Paragraph::new(line).style(Style::default().fg(Color::DarkGray));
    f.render_widget(para, area);
}
