// Theme: ANSI palette only — no RGB hardcodes.
use ratatui::style::Color;

pub const STATUS_COMPLETED: Color = Color::Green;
pub const STATUS_IN_PROGRESS: Color = Color::Cyan;
pub const STATUS_PENDING: Color = Color::DarkGray;
pub const STATUS_FAILED: Color = Color::Red;
pub const STATUS_PARTIAL: Color = Color::Yellow;

pub const GLYPH_COMPLETED: &str = "✓";
pub const GLYPH_IN_PROGRESS: &str = "●";
pub const GLYPH_PENDING: &str = "○";
pub const GLYPH_FAILED: &str = "✗";
pub const GLYPH_PARTIAL: &str = "!";

pub fn status_glyph(status: &str) -> &'static str {
    match status {
        "completed" => GLYPH_COMPLETED,
        "in_progress" => GLYPH_IN_PROGRESS,
        "pending" => GLYPH_PENDING,
        "failed" => GLYPH_FAILED,
        "partial" => GLYPH_PARTIAL,
        _ => "?",
    }
}

pub fn status_color(status: &str) -> Color {
    match status {
        "completed" => STATUS_COMPLETED,
        "in_progress" => STATUS_IN_PROGRESS,
        "pending" => STATUS_PENDING,
        "failed" => STATUS_FAILED,
        "partial" => STATUS_PARTIAL,
        _ => Color::Reset,
    }
}
