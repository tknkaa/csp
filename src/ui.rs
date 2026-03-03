use crate::app::App;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max).collect::<String>())
    } else {
        s.to_string()
    }
}

pub fn ui(f: &mut ratatui::Frame, app: &App) {
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
