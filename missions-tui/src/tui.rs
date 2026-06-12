//! Terminal lifecycle and event loop.
//!
//! `run_app` is the top-level TUI entry point:
//! 1. Installs a panic hook that restores the terminal before printing the panic.
//! 2. Enters raw mode and alternate screen.
//! 3. Runs the sync event loop (poll 250 ms, 1 s tick → snapshot reload).
//! 4. On exit (q / Esc from Selector / panic), restores the terminal.

use crate::app::App;
use crate::registry;
use crate::ui;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io::{self, Stdout},
    time::{Duration, Instant},
};

type Term = Terminal<CrosstermBackend<Stdout>>;

fn setup_terminal() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(term: &mut Term) {
    // Best-effort — ignore errors during cleanup.
    let _ = disable_raw_mode();
    let _ = execute!(term.backend_mut(), LeaveAlternateScreen);
    let _ = term.show_cursor();
}

/// Run the full TUI until the user quits.
pub fn run_app() -> Result<()> {
    // Install panic hook so the terminal is always restored even on panics.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let config_dir = registry::config_dir()?;
    let mut app = App::new(&config_dir);
    let mut terminal = setup_terminal()?;

    let tick_interval = Duration::from_millis(1000);
    let poll_timeout = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        // Draw
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll for key events
        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                // On macOS crossterm emits both Press and Repeat; skip Release.
                if key.kind != KeyEventKind::Release {
                    app.handle_key(key);
                }
            }
        }

        // ~1s tick — reload snapshot
        if last_tick.elapsed() >= tick_interval {
            app.tick_reload();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    restore_terminal(&mut terminal);
    Ok(())
}
