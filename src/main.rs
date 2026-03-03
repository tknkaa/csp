use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Terminal,
};
use std::{
    fs,
    io::{self, BufRead},
    path::PathBuf,
    process::Command,
    time::SystemTime,
};

const PAGE_SIZE: usize = 5;

// ── Session ───────────────────────────────────────────────────────────────────

struct Session {
    id: String,
    start_time: DateTime<Utc>,
    modified: SystemTime,
    first_message: Option<String>,
    message_count: usize,
    cwd: Option<String>,
}

impl Session {
    fn relative_time(&self) -> String {
        let local: DateTime<Local> = self.start_time.into();
        let now = Local::now();
        let diff = now.signed_duration_since(local);
        if diff.num_seconds() < 60 {
            "just now".into()
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            local.format("%Y-%m-%d").to_string()
        }
    }

    fn display_time(&self) -> String {
        let local: DateTime<Local> = self.start_time.into();
        local.format("%m/%d %H:%M").to_string()
    }

    fn preview(&self) -> &str {
        self.first_message.as_deref().unwrap_or("(no messages)")
    }
}

// ── Parse ─────────────────────────────────────────────────────────────────────

fn resolve_events(entry: &PathBuf) -> Option<(String, PathBuf)> {
    if entry.is_dir() {
        let id = entry.file_name()?.to_str()?.to_string();
        let p = entry.join("events.jsonl");
        if p.exists() {
            return Some((id, p));
        }
        let p = entry.join("events.json");
        if p.exists() {
            return Some((id, p));
        }
        None
    } else {
        let name = entry.file_name()?.to_str()?;
        let id = name
            .strip_suffix(".jsonl")
            .or_else(|| name.strip_suffix(".json"))?
            .to_string();
        Some((id, entry.clone()))
    }
}

fn parse_session(id: String, events: &PathBuf) -> Result<Session> {
    let modified = fs::metadata(events)?.modified()?;
    let reader = io::BufReader::new(fs::File::open(events)?);

    let mut start_time: Option<DateTime<Utc>> = None;
    let mut first_message: Option<String> = None;
    let mut message_count = 0usize;
    let mut cwd: Option<String> = None;

    for line in reader.lines().flatten() {
        if line.trim().is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        match v["type"].as_str().unwrap_or("") {
            "session.start" => {
                let ts = v["data"]["startTime"]
                    .as_str()
                    .or_else(|| v["timestamp"].as_str());
                if let Some(ts) = ts {
                    start_time = ts.parse().ok();
                }

                if let Some(c) = v["data"]["context"]["cwd"].as_str() {
                    cwd = Some(c.to_string());
                }
            }
            "user.message" => {
                message_count += 1;
                if first_message.is_none() {
                    if let Some(c) = v["data"]["content"].as_str() {
                        first_message = Some(c.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    let start_time = start_time.unwrap_or_else(|| {
        modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
            .unwrap_or_default()
    });

    Ok(Session {
        id,
        start_time,
        modified,
        first_message,
        message_count,
        cwd,
    })
}

fn load_sessions(dir: &PathBuf) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir).context("cannot read session-state dir")? {
        let path = entry?.path();
        if let Some((id, events)) = resolve_events(&path) {
            if let Ok(s) = parse_session(id, &events) {
                sessions.push(s);
            }
        }
    }
    sessions.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(sessions)
}

// ── App ───────────────────────────────────────────────────────────────────────

struct App {
    sessions: Vec<Session>,
    page: usize,
    cursor: usize,
    total_pages: usize,
}

impl App {
    fn new(sessions: Vec<Session>) -> Self {
        let total_pages = sessions.len().div_ceil(PAGE_SIZE).max(1);
        Self {
            sessions,
            page: 0,
            cursor: 0,
            total_pages,
        }
    }

    fn page_sessions(&self) -> &[Session] {
        let s = self.page * PAGE_SIZE;
        &self.sessions[s..(s + PAGE_SIZE).min(self.sessions.len())]
    }

    fn selected(&self) -> &Session {
        &self.sessions[self.page * PAGE_SIZE + self.cursor]
    }

    fn move_up(&mut self) {
        if self.cursor == 0 {
            if self.page > 0 {
                self.page -= 1;
                self.cursor = PAGE_SIZE - 1;
            }
        } else {
            self.cursor -= 1;
        }
    }

    fn move_down(&mut self) {
        let page_len = self.page_sessions().len();
        if self.cursor + 1 >= page_len {
            if self.page + 1 < self.total_pages {
                self.page += 1;
                self.cursor = 0;
            }
        } else {
            self.cursor += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
            self.cursor = 0;
        }
    }

    fn next_page(&mut self) {
        if self.page + 1 < self.total_pages {
            self.page += 1;
            self.cursor = 0;
        }
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max).collect::<String>())
    } else {
        s.to_string()
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let area = f.area();
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " csp ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "Copilot Session Picker",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{} sessions", app.sessions.len()),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        root[0],
    );

    // Body: list | detail
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root[1]);

    // --- List ---
    let mut list_state = ListState::default();
    list_state.select(Some(app.cursor));

    let dim = Style::default().fg(Color::DarkGray);

    let items: Vec<ListItem> = app
        .page_sessions()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let sel = i == app.cursor;
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        s.display_time(),
                        Style::default()
                            .fg(if sel { Color::Cyan } else { Color::Yellow })
                            .add_modifier(if sel {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    Span::styled(format!("  {}", s.relative_time()), dim),
                ]),
                Line::from(Span::styled(
                    trunc(s.preview(), 42),
                    Style::default()
                        .fg(if sel { Color::White } else { Color::Gray })
                        .add_modifier(if sel {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                )),
                Line::from(Span::styled(format!("{} msgs", s.message_count), dim)),
                Line::from(""),
            ])
        })
        .collect();

    f.render_stateful_widget(
        List::new(items)
            .block(
                Block::default()
                    .title(format!(" Sessions  {}/{} ", app.page + 1, app.total_pages))
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(Style::default().bg(Color::Rgb(28, 38, 52)))
            .highlight_symbol("▶ "),
        body[0],
        &mut list_state,
    );

    // --- Detail ---
    let sel = app.selected();
    let local: DateTime<Local> = sel.start_time.into();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Started  ", dim),
            Span::styled(
                local.format("%Y-%m-%d %H:%M:%S").to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Age      ", dim),
            Span::styled(sel.relative_time(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Messages ", dim),
            Span::styled(
                sel.message_count.to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if let Some(cwd) = &sel.cwd {
        lines.push(Line::from(vec![
            Span::styled("Directory", dim),
            Span::raw("  "),
            Span::styled(cwd.clone(), Style::default().fg(Color::Gray)),
        ]));
    }

    lines.push(Line::from(vec![
        Span::styled("ID       ", dim),
        Span::styled(sel.id.clone(), dim),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "First message",
        dim.add_modifier(Modifier::UNDERLINED),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        sel.preview(),
        Style::default().fg(Color::White),
    )));

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Detail ")
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: false }),
        body[1],
    );

    // Footer
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " ↑↓ ",
                Style::default().fg(Color::Black).bg(Color::DarkGray),
            ),
            Span::raw(" select  "),
            Span::styled(
                " ←→ ",
                Style::default().fg(Color::Black).bg(Color::DarkGray),
            ),
            Span::raw(" page  "),
            Span::styled(
                " Enter ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" resume  "),
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::raw(" quit"),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        root[2],
    );
}

// ── TUI loop ──────────────────────────────────────────────────────────────────

struct Selected {
    id: String,
    cwd: Option<String>,
}

fn run_tui(sessions: Vec<Session>) -> Result<Option<Selected>> {
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

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;
    let session_dir = PathBuf::from(&home).join(".copilot").join("session-state");

    if !session_dir.exists() {
        eprintln!("Session directory not found: {}", session_dir.display());
        std::process::exit(1);
    }

    let sessions = load_sessions(&session_dir)?;
    if sessions.is_empty() {
        eprintln!("No sessions found.");
        std::process::exit(1);
    }

    let Some(sel) = run_tui(sessions)? else {
        return Ok(());
    };

    // cd してから exec
    let work_dir = sel
        .cwd
        .as_deref()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(&home));

    eprintln!("cd {}", work_dir.display());
    eprintln!("Resuming {}…", &sel.id[..8.min(sel.id.len())]);

    std::env::set_current_dir(&work_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new("copilot")
            .arg(format!("--resume={}", sel.id))
            .exec();
        eprintln!("exec failed: {}", err);
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        Command::new("copilot")
            .arg(format!("--resume={}", sel.id))
            .status()?;
        Ok(())
    }
}
