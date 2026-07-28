pub mod matrix;
mod matrix_scores;
mod big_text;
mod chess_game;
mod game;
mod lang;
mod ledger_store;
mod mp;
mod paradox;
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

fn print_help() {
    println!("bet — terminal games + multiplayer virtual bets");
    println!();
    println!("Usage: bet [COMMAND] [options]");
    println!();
    println!("Single-player (TUI hub if no command):");
    println!("  hangman | tictactoe | chess | pong | matrix | paradox");
    println!("  movies | series | manga | books | anime | cartoon | games | music");
    println!();
    println!("Multiplayer (virtual points, tic-tac-toe):");
    println!("  host [--stake N] [--port P] [--name ID] [--bind ADDR]");
    println!("  join CODE [--stake N] [--addr HOST:PORT] [--name ID]");
    println!("  balance              Show local virtual ledger");
    println!();
    println!("Options:");
    println!("  --stake N            Points to wager (default 10)");
    println!("  --port P             Host listen port (default {})", mp::DEFAULT_PORT);
    println!("  --addr HOST:PORT     Guest connect address (default 127.0.0.1:{})", mp::DEFAULT_PORT);
    println!("  --name ID            Player id (default $BET_PLAYER or $USER)");
    println!("  --bind ADDR          Host bind address (default 0.0.0.0)");
    println!("  -h, --help           Help");
    println!("  -v, --version        Version");
    println!();
    println!("Example:");
    println!("  bet host --stake 10 --name alice");
    println!("  bet join ABC123 --stake 10 --name bob");
}

fn parse_flag(args: &[String], name: &str) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == name {
            return args.get(i + 1).cloned();
        }
        if let Some(rest) = args[i].strip_prefix(&format!("{name}=")) {
            return Some(rest.to_string());
        }
        i += 1;
    }
    None
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 {
        let cmd = args[1].to_lowercase();
        if cmd == "--help" || cmd == "-h" || cmd == "help" {
            print_help();
            return Ok(());
        }
        if cmd == "--version" || cmd == "-v" {
            println!("bet {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        if cmd == "balance" || cmd == "balances" {
            mp::print_balances();
            return Ok(());
        }
        if cmd == "host" {
            let stake = parse_flag(&args, "--stake")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            let port = parse_flag(&args, "--port")
                .and_then(|s| s.parse().ok())
                .unwrap_or(mp::DEFAULT_PORT);
            let name = parse_flag(&args, "--name").unwrap_or_else(ledger_store::default_player_id);
            let bind = parse_flag(&args, "--bind").unwrap_or_else(|| "0.0.0.0".into());
            return mp::run_host(mp::HostOpts {
                stake,
                port,
                name,
                bind,
            });
        }
        if cmd == "join" {
            let code = args
                .get(2)
                .filter(|s| !s.starts_with('-'))
                .cloned()
                .ok_or("usage: bet join CODE [--stake N] [--addr HOST:PORT]")?;
            let stake = parse_flag(&args, "--stake")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            let addr = parse_flag(&args, "--addr")
                .unwrap_or_else(|| format!("127.0.0.1:{}", mp::DEFAULT_PORT));
            let name = parse_flag(&args, "--name").unwrap_or_else(ledger_store::default_player_id);
            return mp::run_join(mp::JoinOpts {
                stake,
                addr,
                room_code: code,
                name,
            });
        }
    }

    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        eprintln!("Error: 'bet' requires a TTY terminal to run (or use host/join/balance).");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;

    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = app.run(&mut terminal);

    drop(_guard);

    if let Err(err) = res {
        eprintln!("Error running application: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}
