use humansize::{BINARY, format_size};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::app::AppState;
use crate::fs_tree::NodeKind;

const BAR_WIDTH: usize = 14;

pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let visible_height = area.height.saturating_sub(2) as usize; // subtract borders

    if state.flat_list.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Scanning... ");
        f.render_widget(block, area);
        return;
    }

    let total_width = area.width.saturating_sub(2) as usize; // subtract borders
    // Layout: [indent+icon+checkbox+name] [10 size] [1 space] [bar+2] [1 space]
    let fixed = 10 + 1 + BAR_WIDTH + 2 + 1;
    let name_col = total_width.saturating_sub(fixed);

    let items: Vec<ListItem> = state
        .flat_list
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(visible_height)
        .map(|(i, item)| {
            let is_cursor = i == state.cursor;
            build_list_item(item, is_cursor, name_col)
        })
        .collect();

    let selected_count = state.selected_paths.len();
    let title = if selected_count > 0 {
        format!(" Files  ({} selected) ", selected_count)
    } else {
        " Files ".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, area);
}

fn build_list_item(
    item: &crate::fs_tree::FlatItem,
    is_cursor: bool,
    name_col: usize,
) -> ListItem<'static> {
    let indent = "  ".repeat(item.depth);

    let icon = match (&item.kind, item.has_children, item.expanded) {
        (NodeKind::Directory, true, true) => "▼ ",
        (NodeKind::Directory, true, false) => "▶ ",
        (NodeKind::Directory, false, _) => "  ",
        (NodeKind::File, _, _) => "  ",
    };

    let checkbox = if item.selected { "[x] " } else { "[ ] " };

    let prefix_len = indent.len() + icon.len() + checkbox.len();
    let avail_name = name_col.saturating_sub(prefix_len);
    let name = if item.name.len() > avail_name && avail_name > 3 {
        format!("{}...", &item.name[..avail_name.saturating_sub(3)])
    } else {
        item.name.clone()
    };

    let name_padded = format!("{}{}{}{:<width$}", indent, checkbox, icon, name, width = avail_name);

    let size_str = format!("{:>10}", format_size(item.size, BINARY));

    let pct = if item.parent_size > 0 {
        (item.size as f64 / item.parent_size as f64).min(1.0)
    } else {
        0.0
    };
    let filled = (pct * BAR_WIDTH as f64) as usize;
    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(BAR_WIDTH - filled));

    let base_style = if is_cursor {
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else if item.selected {
        Style::default().fg(Color::Yellow)
    } else {
        match item.kind {
            NodeKind::Directory => Style::default().fg(Color::Cyan),
            NodeKind::File => Style::default(),
        }
    };

    let bar_color = if is_cursor { Color::White } else { Color::Green };

    let line = Line::from(vec![
        Span::styled(name_padded, base_style),
        Span::styled(size_str, if is_cursor { base_style } else { Style::default().fg(Color::Magenta) }),
        Span::styled(" ", base_style),
        Span::styled(bar, if is_cursor { base_style } else { Style::default().fg(bar_color) }),
    ]);

    ListItem::new(line)
}
