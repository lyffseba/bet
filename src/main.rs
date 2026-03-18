pub mod matrix;
mod matrix_scores;
mod big_text;
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        let cmd = args[1].to_lowercase();
        if cmd == "--help" || cmd == "-h" {
            println!("bet - 3A Carmack-level terminal multiplexer");
            println!("Usage: bet [COMMAND]");
            println!("Commands:");
            println!("  hangman      Play Hangman");
            println!("  tictactoe    Play Tic-Tac-Toe");
            println!("  chess        Play Chess");
            println!("  pong         Play Pong");
            println!("  matrix       Enter The Matrix");
            println!("  movies       Get Movie Recommendations");
            println!("  series       Get TV Series Recommendations");
            println!("  manga        Get Manga Recommendations");
            println!("  books        Get Book Recommendations");
            println!("  anime        Get Anime Recommendations");
            println!("  cartoon      Get Cartoon Recommendations");
            println!("  games        Get Video Game Recommendations");
            println!("  music        Get Music Recommendations");
            return Ok(());
        }
        if cmd == "--version" || cmd == "-v" {
            println!("bet v1.0.0");
            return Ok(());
        }
    }

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
