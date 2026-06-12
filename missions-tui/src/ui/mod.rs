pub mod dashboard;
pub mod features;
pub mod selector;
pub mod theme;
pub mod widgets;
pub mod workers;

use crate::app::{App, Screen};
use ratatui::Frame;

/// Top-level render dispatcher — routes to the correct screen renderer.
pub fn draw(f: &mut Frame, app: &App) {
    match &app.screen {
        Screen::Selector => selector::draw_selector(f, app),
        Screen::Dashboard => dashboard::draw_dashboard(f, app),
        Screen::Features => features::draw_features(f, app),
        Screen::FeatureDetail(id) => {
            let id = id.clone();
            features::draw_feature_detail(f, app, &id);
        }
        Screen::Workers => workers::draw_workers(f, app),
    }
}
