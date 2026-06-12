use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

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
