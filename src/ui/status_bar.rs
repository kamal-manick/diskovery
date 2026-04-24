use humansize;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{AppMode, AppState};

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let (line1, line2) = match state.mode {
        AppMode::Scanning => {
            let (dirs_done, bytes_done) = state.scan_progress;
            let scanning_line = if dirs_done == 0 {
                " Scanning...  [Q] Quit".to_string()
            } else {
                format!(
                    " Scanning... {dirs_done} top-level items done, {} scanned so far  [Q] Quit",
                    humansize::format_size(bytes_done, humansize::BINARY)
                )
            };
            (scanning_line, String::new())
        }
        AppMode::Browsing => {
            let sel = state.selected_paths.len();
            let sel_str = if sel > 0 {
                format!("  |  {} selected", sel)
            } else {
                String::new()
            };
            let msg = state.status_msg.as_deref().unwrap_or("").to_string();
            (
                format!(
                    " [↑↓] Navigate  [Enter/→] Expand  [←] Collapse  [Space] Select  [D] Delete  [R] Rescan  [Q] Quit{}",
                    sel_str
                ),
                msg,
            )
        }
        AppMode::Confirming => (
            " [Tab/←/→] Switch button  [Enter] Confirm  [Esc] Cancel".to_string(),
            String::new(),
        ),
    };

    let hint_style = Style::default().fg(Color::DarkGray);
    let msg_style = Style::default().fg(Color::Yellow);

    let text = vec![
        Line::from(Span::styled(line1, hint_style)),
        Line::from(Span::styled(line2, msg_style)),
    ];

    f.render_widget(Paragraph::new(text), area);
}
