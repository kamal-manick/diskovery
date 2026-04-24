use humansize::{BINARY, format_size};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge};

use crate::app::DriveInfo;

pub fn render(f: &mut Frame, area: Rect, drive: &DriveInfo) {
    if drive.total_bytes == 0 {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Drive info unavailable ");
        f.render_widget(block, area);
        return;
    }

    let ratio = drive.used_bytes as f64 / drive.total_bytes as f64;
    let ratio = ratio.clamp(0.0, 1.0);

    let bar_color = if ratio > 0.90 {
        Color::Red
    } else if ratio > 0.75 {
        Color::Yellow
    } else {
        Color::Green
    };

    let title = format!(
        " {}  Total: {}   Used: {}   Free: {}   ({:.1}%) ",
        if drive.mount.is_empty() { "Drive".to_string() } else { drive.mount.clone() },
        format_size(drive.total_bytes, BINARY),
        format_size(drive.used_bytes, BINARY),
        format_size(drive.free_bytes, BINARY),
        ratio * 100.0,
    );

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(Style::default().fg(bar_color).bg(Color::DarkGray))
        .ratio(ratio);

    f.render_widget(gauge, area);
}
