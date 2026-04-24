use humansize::{BINARY, format_size};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::AppState;
use crate::ui::centered_rect;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let dialog_area = centered_rect(60, 70, area);

    // Clamp height based on content
    let items_shown = state.confirm_targets.len().min(10);
    let needed_height = (items_shown + 8) as u16;
    let dialog_area = if dialog_area.height > needed_height {
        Rect {
            y: dialog_area.y + (dialog_area.height - needed_height) / 2,
            height: needed_height,
            ..dialog_area
        }
    } else {
        dialog_area
    };

    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm Deletion ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let chunks = Layout::vertical([
        Constraint::Length(1),                           // blank
        Constraint::Length(1),                           // header
        Constraint::Min(1),                              // item list
        Constraint::Length(1),                           // blank
        Constraint::Length(1),                           // total
        Constraint::Length(1),                           // blank
        Constraint::Length(1),                           // buttons
        Constraint::Length(1),                           // blank
    ])
    .split(inner);

    // Header
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {} item(s) will be permanently deleted:", state.confirm_targets.len()),
            Style::default().fg(Color::White),
        ))),
        chunks[1],
    );

    // Item list
    let items_area = chunks[2];
    let max_items = items_area.height as usize;
    let mut lines: Vec<Line> = state
        .confirm_targets
        .iter()
        .take(max_items.saturating_sub(if state.confirm_targets.len() > max_items { 1 } else { 0 }))
        .map(|(path, size)| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            Line::from(vec![
                Span::styled("   • ", Style::default().fg(Color::Yellow)),
                Span::styled(name, Style::default().fg(Color::White)),
                Span::styled(
                    format!("  ({})", format_size(*size, BINARY)),
                    Style::default().fg(Color::Magenta),
                ),
            ])
        })
        .collect();

    if state.confirm_targets.len() > max_items {
        let remaining = state.confirm_targets.len() - (max_items - 1);
        lines.push(Line::from(Span::styled(
            format!("   ... and {} more", remaining),
            Style::default().fg(Color::DarkGray),
        )));
    }

    f.render_widget(Paragraph::new(lines), items_area);

    // Total
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Total space freed: ", Style::default().fg(Color::White)),
            Span::styled(
                format_size(state.confirm_total_bytes, BINARY),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])),
        chunks[4],
    );

    // Buttons
    let yes_style = if state.confirm_focus_yes {
        Style::default()
            .bg(Color::Red)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let cancel_style = if !state.confirm_focus_yes {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let buttons = Line::from(vec![
        Span::raw("  "),
        Span::styled("  Yes, Delete  ", yes_style),
        Span::raw("    "),
        Span::styled("  Cancel  ", cancel_style),
    ]);

    f.render_widget(Paragraph::new(buttons), chunks[6]);
}
