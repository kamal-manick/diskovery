pub mod drive_bar;
pub mod tree_view;
pub mod status_bar;
pub mod confirm_dialog;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::app::{AppMode, AppState};

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let (bar_area, tree_area, status_area) = build_layout(area);

    drive_bar::render(f, bar_area, &state.drive);
    tree_view::render(f, tree_area, state);
    status_bar::render(f, status_area, state);

    if state.mode == AppMode::Confirming {
        confirm_dialog::render(f, area, state);
    }
}

pub fn build_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(area);
    (chunks[0], chunks[1], chunks[2])
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vert = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vert[1])[1]
}
