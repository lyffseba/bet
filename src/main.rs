mod chess_game;
mod game;
mod lang;
mod pong;
mod tictactoe;
mod ui;
mod wordlist;

use std::error::Error;
use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use ui::App;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, crossterm::cursor::Show);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    use std::io::IsTerminal;
    // Basic terminal capability check (are we a tty?)
    if !io::stdout().is_terminal() {
        eprintln!("Error: 'bet' requires a TTY terminal to run.");
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;

    // Ensure cleanup happens even on panic or early return
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run the app
    let mut app = App::new();
    let res = app.run(&mut terminal);

    // Drop guard early to restore terminal before printing errors
    drop(_guard);

    if let Err(err) = res {
        eprintln!("Error running application: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}
