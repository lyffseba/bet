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
    println!("Multiplayer (virtual points):");
    println!("  host [--game ttt|hangman] [--stake N] [--port P] [--name ID]");
    println!("       [--bind ADDR] [--code CODE] [--word WORD] [--seed N]");
    println!("  join CODE|CODE@HOST:PORT [--game ttt|hangman] [--stake N]");
    println!("       [--addr HOST:PORT] [--name ID]");
    println!("  balance              Show local virtual ledger");
    println!();
    println!("Options:");
    println!("  --game ttt|hangman   Multiplayer game (default ttt)");
    println!("  --stake N            Points to wager (default 10)");
    println!("  --port P             Host listen port (default {})", mp::DEFAULT_PORT);
    println!(
        "  --addr HOST:PORT     Guest connect address (default 127.0.0.1:{})",
        mp::DEFAULT_PORT
    );
    println!("  --name ID            Player id (default $BET_PLAYER or $USER)");
    println!("  --bind ADDR          Host bind address (default 0.0.0.0)");
    println!("  --code CODE          Fixed room code (4–8 alnum, tests)");
    println!("  --word WORD          Hangman host: pin the secret word (tests)");
    println!("  --seed N             Hangman host: pick word from default bank");
    println!("  -h, --help           Help");
    println!("  -v, --version        Version");
    println!();
    println!("Env:");
    println!("  BET_CONFIG_DIR       Ledger directory");
    println!("  BET_PLAYER           Default player id");
    println!("  BET_MOVES            Scripted moves e.g. 0,3,1,4,2 or B,E,T");
    println!();
    println!("Example (two terminals):");
    println!("  # Terminal 1: start the host");
    println!("  bet host --stake 10 --name alice");
    println!("  # Terminal 2: join using the room code printed in BET_READY");
    println!("  bet join JEST33@127.0.0.1:7733 --stake 10 --name bob");
    println!("Hangman:");
    println!("  bet host --game hangman --word BET --stake 10 --name alice");
    println!("  bet join CODE@127.0.0.1:7733 --game hangman --stake 10 --name bob");
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
            let code = parse_flag(&args, "--code");
            let game = parse_flag(&args, "--game")
                .map(|s| s.parse())
                .transpose()
                .map_err(|e: String| e)?
                .unwrap_or_default();
            let word = parse_flag(&args, "--word");
            let seed = parse_flag(&args, "--seed").and_then(|s| s.parse().ok());
            return mp::run_host(mp::HostOpts {
                stake,
                port,
                name,
                bind,
                code,
                game,
                word,
                seed,
            });
        }
        if cmd == "join" {
            let raw = args
                .get(2)
                .filter(|s| !s.starts_with('-'))
                .cloned()
                .ok_or("usage: bet join CODE|CODE@HOST:PORT [--stake N] [--game ttt|hangman]")?;
            let (code, addr_from_target) = mp::parse_join_target(&raw);
            let stake = parse_flag(&args, "--stake")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            let addr = parse_flag(&args, "--addr")
                .or(addr_from_target)
                .unwrap_or_else(|| format!("127.0.0.1:{}", mp::DEFAULT_PORT));
            let name = parse_flag(&args, "--name").unwrap_or_else(ledger_store::default_player_id);
            let game = parse_flag(&args, "--game")
                .map(|s| s.parse())
                .transpose()
                .map_err(|e: String| e)?
                .unwrap_or_default();
            return mp::run_join(mp::JoinOpts {
                stake,
                addr,
                room_code: code,
                name,
                game,
            });
        }
    }

    use std::io::IsTerminal;
    if !io::stdout().is_terminal() {
        eprintln!("Error: 'bet' requires a TTY (or use host/join/balance).");
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
