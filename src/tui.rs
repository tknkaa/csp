use crate::{app::App, session::Session, ui::ui};
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub struct Selected {
    pub id: String,
    pub cwd: Option<String>,
}

pub fn run_tui(sessions: Vec<Session>) -> Result<Option<Selected>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(sessions);

    let result = loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _)
                | (KeyCode::Esc, _)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break None,

                (KeyCode::Up | KeyCode::Char('k'), _) => app.move_up(),
                (KeyCode::Down | KeyCode::Char('j'), _) => app.move_down(),
                (KeyCode::Left | KeyCode::Char('h'), _) => app.prev_page(),
                (KeyCode::Right | KeyCode::Char('l'), _) => app.next_page(),

                (KeyCode::Enter, _) => {
                    let s = app.selected();
                    break Some(Selected {
                        id: s.id.clone(),
                        cwd: s.cwd.clone(),
                    });
                }
                _ => {}
            }
        }
    };

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(result)
}
