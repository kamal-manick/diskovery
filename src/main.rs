mod app;
mod fs_tree;
mod scanner;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::{AppMode, AppState};

fn main() -> Result<()> {
    let root_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\"))
        });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Always restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let mut state = AppState::new(root_path);
    let result = run_app(&mut terminal, &mut state);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: &mut AppState) -> Result<()> {
    loop {
        state.poll_scan();

        terminal.draw(|f| ui::render(f, state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(state, key.code, key.modifiers, terminal);
                }
            }
        }

        if state.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(
    state: &mut AppState,
    code: KeyCode,
    modifiers: KeyModifiers,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) {
    let _ = modifiers; // suppress unused warning; may use Ctrl in future
    let visible_height = terminal.size().map(|s| s.height as usize).unwrap_or(24).saturating_sub(7);

    match state.mode {
        AppMode::Scanning => {
            if matches!(code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                state.should_quit = true;
            }
        }

        AppMode::Browsing => match code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                state.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => state.cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => state.cursor_down(visible_height),
            KeyCode::PageUp => state.page_up(visible_height),
            KeyCode::PageDown => state.page_down(visible_height),
            KeyCode::Home => {
                state.cursor = 0;
                state.scroll_offset = 0;
            }
            KeyCode::End => {
                let max = state.flat_list.len().saturating_sub(1);
                state.cursor = max;
                if max >= visible_height {
                    state.scroll_offset = max + 1 - visible_height;
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => state.toggle_expand(),
            KeyCode::Left | KeyCode::Char('h') => state.collapse_or_parent(),
            KeyCode::Char(' ') => state.toggle_select(),
            KeyCode::Char('a') | KeyCode::Char('A') => state.select_all(),
            KeyCode::Char('d') | KeyCode::Char('D') => state.open_confirm(),
            KeyCode::Char('r') | KeyCode::Char('R') => {
                state.status_msg = None;
                state.start_scan();
            }
            _ => {}
        },

        AppMode::Confirming => match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => state.cancel_confirm(),
            KeyCode::Tab | KeyCode::Left | KeyCode::Right
            | KeyCode::Char('h') | KeyCode::Char('l') => {
                state.confirm_focus_yes = !state.confirm_focus_yes;
            }
            KeyCode::Enter => {
                if state.confirm_focus_yes {
                    state.execute_delete();
                } else {
                    state.cancel_confirm();
                }
            }
            _ => {}
        },
    }
}
