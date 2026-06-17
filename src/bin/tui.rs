use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::time::Duration;

use calc::{evaluate, fast_two, fmt_op};

struct App {
    input: Vec<String>,
    cursor: usize,
    scroll: usize,
    output: Vec<String>,
    status: String,
    cursor_pos: usize,
}

impl App {
    fn new() -> Self {
        Self {
            input: vec!["".to_string()],
            cursor: 0,
            scroll: 0,
            output: vec!["ZEISX v1.0 — type expressions, Ctrl+R to run, Ctrl+C to quit".to_string(), "".to_string()],
            status: "READY".to_string(),
            cursor_pos: 0,
        }
    }

    fn run(&mut self) {
        let line = match self.input.get(self.cursor) { Some(l) => l, None => return };
        if line.trim().is_empty() { return; }

        match evaluate(line.trim()) {
            Ok(n) => {
                if let Some(p) = fast_two(line.trim()) {
                    self.output.push(format!("{} {} {} = {}", p.0, fmt_op(p.1), p.2, n));
                } else {
                    self.output.push(format!("  => {}", n));
                }
                self.status = "OK".to_string();
            }
            Err(e) => {
                self.output.push(format!("error: {}", e));
                self.status = "ERROR".to_string();
            }
        }
        self.input.push("".to_string());
        self.cursor += 1;
        self.cursor_pos = 0;
        if self.output.len() > 500 { self.output.drain(0..100); }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let tick_rate = Duration::from_millis(100);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(f.size());

            let editor_block = Block::default()
                .title(" ZEISX EDITOR ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.status == "ERROR" { Color::Red } else { Color::Green }));

            let mut lines: Vec<Line> = Vec::new();
            for (i, line) in app.input.iter().enumerate() {
                let prefix = format!("{:3} │ ", i + 1);
                if i == app.cursor {
                    let pos = app.cursor_pos.min(line.len());
                    let before = &line[..pos];
                    let after = &line[pos..];
                    lines.push(Line::from(vec![
                        Span::raw(prefix),
                        Span::raw(before),
                        Span::styled("█", Style::default().fg(Color::Cyan)),
                        Span::raw(after),
                    ]));
                } else {
                    lines.push(Line::from(vec![Span::raw(prefix), Span::raw(line.clone())]));
                }
            }

            let editor = Paragraph::new(lines)
                .block(editor_block)
                .wrap(Wrap { trim: false })
                .scroll((app.scroll as u16, 0));
            f.render_widget(editor, chunks[0]);

            let bottom_title = match app.status.as_str() {
                "ERROR" => Span::styled(format!(" [{}] ", app.status), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                _ => Span::styled(format!(" [{}] ", app.status), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            };

            let out_block = Block::default()
                .title(Span::styled(" OUTPUT ", Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .title_bottom(bottom_title);

            let out = app.output.last().map(|s| s.as_str()).unwrap_or("");
            let out_style = if out.starts_with("error") { Color::Red } else if out.starts_with("  =>") { Color::Green } else { Color::Gray };
            let out_lines = vec![
                Line::styled(Span::raw(out), Style::default().fg(out_style)),
                Line::raw(""),
            ];
            let out_para = Paragraph::new(out_lines).block(out_block).wrap(Wrap { trim: false });
            f.render_widget(out_para, chunks[1]);
        })?;

        if event::poll(tick_rate)? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char(c) => {
                        if app.cursor < app.input.len() {
                            let line = &mut app.input[app.cursor];
                            line.insert(app.cursor_pos, c);
                            app.cursor_pos += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if app.cursor_pos > 0 && app.cursor < app.input.len() {
                            let line = &mut app.input[app.cursor];
                            line.remove(app.cursor_pos - 1);
                            app.cursor_pos -= 1;
                        } else if app.cursor_pos == 0 && app.cursor > 0 {
                            let prev = app.input.remove(app.cursor - 1);
                            app.cursor -= 1;
                            app.cursor_pos = app.input[app.cursor].len();
                            app.input[app.cursor].push_str(&prev);
                        }
                    }
                    KeyCode::Delete => {
                        if app.cursor < app.input.len() {
                            let line = &mut app.input[app.cursor];
                            if app.cursor_pos < line.len() { line.remove(app.cursor_pos); }
                        }
                    }
                    KeyCode::Enter => app.run(),
                    KeyCode::Up => if app.cursor > 0 { app.cursor -= 1; app.cursor_pos = app.cursor_pos.min(app.input[app.cursor].len()); },
                    KeyCode::Down => if app.cursor < app.input.len() - 1 { app.cursor += 1; app.cursor_pos = app.cursor_pos.min(app.input[app.cursor].len()); },
                    KeyCode::Left => if app.cursor_pos > 0 { app.cursor_pos -= 1; },
                    KeyCode::Right => if app.cursor < app.input.len() { app.cursor_pos = (app.cursor_pos + 1).min(app.input[app.cursor].len()); },
                    KeyCode::Home => { app.cursor_pos = 0; }
                    KeyCode::End => { app.cursor_pos = app.input[app.cursor].len(); }
                    KeyCode::PageUp => { app.scroll = app.scroll.saturating_sub(10); }
                    KeyCode::PageDown => { app.scroll = app.scroll.saturating_add(10); }
                    KeyCode::Ctrl(c) => match c {
                        'r' | 'R' => { app.status = "RUNNING".to_string(); app.run(); app.status = "OK".to_string(); }
                        'c' => break,
                        's' => { app.status = "SAVED".to_string(); }
                        _ => {}
                    },
                    KeyCode::Esc => { app.status = "READY".to_string(); }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
