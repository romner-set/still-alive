use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*, symbols::border};

use std::sync::{Arc, Mutex};
use std::sync::atomic::{Ordering, AtomicBool};
use std::io::{self, stdout};

// Modules
mod threads;

// Constants
const BORDER_SET_L: border::Set = border::Set {
    top_left: "-",
    top_right: " ",
    bottom_left: "-",
    bottom_right: " ",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};
const BORDER_SET_R: border::Set = border::Set {
    top_left: " ",
    top_right: " ",
    bottom_left: " ",
    bottom_right: " ",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

const COLOR: Color = if cfg!(target_os = "linux") {Color::Rgb(200, 162, 45)}
                     else {Color::Yellow}; // Others may not support 24-bit color and are less
                                           // likely to have non-standard yellow

// Main
fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Spawn threads
    let left_cursor_visible = Arc::new(AtomicBool::new(false));
    let right_cursor_visible = Arc::new(AtomicBool::new(false));
    threads::cursor(Arc::clone(&left_cursor_visible));
    threads::cursor(Arc::clone(&right_cursor_visible));

    let left_panel = Arc::new(Mutex::new(String::new()));
    let right_panel = Arc::new(Mutex::new(String::new()));
    let ascii = Arc::new(Mutex::new(String::new()));
    threads::main(Arc::clone(&left_panel), Arc::clone(&right_panel), Arc::clone(&ascii));

    // Render loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| {
            let horizontal = Layout::new(
                Direction::Horizontal,
                [Constraint::Min(0), Constraint::Length(50), Constraint::Length(51), Constraint::Min(0)],
            ).split(f.size());

            let left = Layout::new(
                Direction::Vertical,
                [Constraint::Length(36)],
            ).split(horizontal[1]);

            let right = Layout::new(
                Direction::Vertical,
                [Constraint::Length(18), Constraint::Length(20)],
            ).split(horizontal[2]);

            // LEFT
            f.render_widget(
                {
                    let mut text = (*left_panel.lock().unwrap()).clone();
                    if left_cursor_visible.load(Ordering::Relaxed) {text.push('_')};
                    Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).border_set(BORDER_SET_L))
                        .style(Style::new().fg(COLOR))
                },
                left[0],
            );

            // RIGHT
            f.render_widget(
                {
                    let mut text = (*right_panel.lock().unwrap()).clone();
                    if right_cursor_visible.load(Ordering::Relaxed) {text.push('_')};

                    let line_count = text.as_bytes().iter().filter(|&&c| c == b'\n').count();
                    if line_count < 15 {text.insert_str(0, &"\n".repeat(15-line_count));}

                    Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).border_set(BORDER_SET_R))
                        .scroll((if line_count > 15 {line_count as u16-15} else {0}, 0))
                        .style(Style::new().fg(COLOR))
                },
                right[0],
            );
            f.render_widget(
                Paragraph::new((*ascii.lock().unwrap()).clone()).style(Style::new().fg(COLOR)),
                right[1],
            );
        })?;

        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
