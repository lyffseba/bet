//! Multiplayer host/join over TCP (newline-delimited JSON).
//!
//! - Scripted moves: `BET_MOVES=0,3,1,4,2` (ttt) or `BET_MOVES=B,E,T` (hangman)
//! - Machine line: `BET_READY room=CODE port=P stake=N proto=1 game=ttt|hangman`
//! - TTT: empty cells show indices 0–8. Hangman: type a letter; `q` resigns.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use bet_core::rng::XorShift64;
use bet_protocol::hangman_room::DEFAULT_MAX_ATTEMPTS;
use bet_protocol::host::HostRoom;
use bet_protocol::msg::{
    decode_client_line, decode_server_line, encode_line, ClientMsg, GameKind, Role, ServerMsg,
    PROTOCOL_VERSION,
};
use bet_protocol::room::{normalize_code, Phase, RoomError, RoomEvent};
use crate::wordlist::ENGLISH_MOVIES;

use crate::ledger_store::{
    default_player_id, ledger_path, load_ledger, merge_balances, save_ledger,
};

pub const DEFAULT_PORT: u16 = 7733;

pub struct HostOpts {
    pub stake: i64,
    pub port: u16,
    pub name: String,
    pub bind: String,
    /// Fixed room code (tests); random if None.
    pub code: Option<String>,
    pub game: GameKind,
    /// Deterministic hangman word (tests). Ignored for ttt.
    pub word: Option<String>,
    /// Seed for hangman word pick when `--word` is absent.
    pub seed: Option<u64>,
}

pub struct JoinOpts {
    pub stake: i64,
    pub addr: String,
    pub room_code: String,
    pub name: String,
    pub game: GameKind,
}

fn gen_room_code() -> String {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let mut rng = XorShift64::new(seed);
    const ALPH: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    (0..6)
        .map(|_| ALPH[rng.gen_range(ALPH.len())] as char)
        .collect()
}

fn move_queue() -> &'static Mutex<VecDeque<String>> {
    use std::sync::OnceLock;
    static Q: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
    Q.get_or_init(|| {
        let mut q = VecDeque::new();
        if let Ok(raw) = std::env::var("BET_MOVES") {
            for part in raw.split(|c: char| c == ',' || c.is_whitespace()) {
                let t = part.trim();
                if !t.is_empty() {
                    q.push_back(t.to_string());
                }
            }
        }
        Mutex::new(q)
    })
}

fn print_board(board: &[char; 9]) {
    println!();
    for row in 0..3 {
        let i = row * 3;
        println!(
            "  {} | {} | {}",
            cell_label(board[i], i),
            cell_label(board[i + 1], i + 1),
            cell_label(board[i + 2], i + 2)
        );
        if row < 2 {
            println!(" ---+---+---");
        }
    }
    println!();
}

fn cell_label(c: char, idx: usize) -> char {
    if c == '.' {
        char::from_digit(idx as u32, 10).unwrap_or('?')
    } else {
        c
    }
}

/// Compact gallows + spaced mask. Stage = misses used (0..=6).
fn print_hangman(display: &str, attempts_left: u32, max_attempts: u32) {
    const ART: [&str; 7] = [
        "  +---+\n  |   |\n      |\n      |\n      |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n      |\n      |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n  |   |\n      |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n /|   |\n      |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n /|\\  |\n      |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n /|\\  |\n /    |\n      |\n ======",
        "  +---+\n  |   |\n  O   |\n /|\\  |\n / \\  |\n      |\n ======",
    ];
    let used = max_attempts.saturating_sub(attempts_left);
    let stage = used.min(6) as usize;
    println!();
    println!("{}", ART[stage]);
    let spaced: String = display
        .chars()
        .map(|c| if c == '_' { '_' } else { c })
        .flat_map(|c| [c, ' '])
        .collect();
    println!("  {}", spaced.trim_end());
    println!("  attempts={attempts_left}/{max_attempts}");
}

fn apply_server_msg(msg: &ServerMsg, my_role: &mut Option<Role>) -> bool {
    match msg {
        ServerMsg::Welcome {
            role,
            stake,
            room_code,
            match_id,
            player_id,
            proto,
            game,
        } => {
            *my_role = Some(*role);
            println!(
                "Welcome {player_id} as {role:?} | game={game} | stake={stake} | room={room_code} | match={match_id} | proto={proto}"
            );
            false
        }
        ServerMsg::PeerJoined { player_id } => {
            println!("Peer joined: {player_id}");
            false
        }
        ServerMsg::State {
            board,
            current,
            status,
            pot,
            your_turn,
            ..
        } => {
            print_board(board);
            println!("status={status} current={current:?} pot={pot} your_turn={your_turn}");
            if *your_turn {
                println!("hint: type empty cell index 0-8 (shown on board), or q to resign");
            }
            false
        }
        ServerMsg::HangmanState {
            display,
            guessed,
            attempts_left,
            max_attempts,
            current,
            status,
            pot,
            your_turn,
            last_guess,
            last_hit,
        } => {
            print_hangman(display, *attempts_left, *max_attempts);
            println!("  guessed=[{guessed}]");
            if let Some(g) = last_guess {
                let hit = last_hit.unwrap_or(false);
                println!("  last={g} ({})", if hit { "hit" } else { "miss" });
            }
            println!("status={status} current={current:?} pot={pot} your_turn={your_turn}");
            if *your_turn {
                println!("hint: type a letter, or q to resign");
            }
            false
        }
        ServerMsg::MatchEnded {
            winner,
            pot,
            winner_id,
            host_id,
            guest_id,
            balance_host,
            balance_guest,
            word,
        } => {
            println!("=== MATCH ENDED ===");
            println!("winner={winner:?} winner_id={winner_id:?} pot={pot}");
            println!("host={host_id}={balance_host} guest={guest_id:?}={balance_guest}");
            if let Some(w) = word {
                println!("word={w}");
            }
            true
        }
        ServerMsg::Error { message } => {
            eprintln!("error: {message}");
            false
        }
        ServerMsg::Pong => false,
    }
}

enum Input {
    Place(u8),
    Guess(char),
    Resign,
}

fn scripted_active() -> bool {
    std::env::var("BET_MOVES").is_ok()
}

fn next_scripted() -> Option<String> {
    move_queue().lock().ok().and_then(|mut q| q.pop_front())
}

fn is_resign(t: &str) -> bool {
    t.eq_ignore_ascii_case("q") || t.eq_ignore_ascii_case("resign")
}

fn parse_ttt_token(t: &str) -> Option<Input> {
    if is_resign(t) {
        return Some(Input::Resign);
    }
    t.parse::<u8>().ok().filter(|&n| n < 9).map(Input::Place)
}

fn parse_hangman_token(t: &str) -> Option<Input> {
    if is_resign(t) {
        return Some(Input::Resign);
    }
    let mut chars = t.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Some(Input::Guess(c)),
        _ => None,
    }
}

/// Scripted mode (`BET_MOVES` set, even empty) never blocks on stdin.
/// Exhausted / invalid script → resign so e2e cannot hang a host waiting
/// for a turn it does not need.
fn read_input(game: GameKind) -> Input {
    if scripted_active() {
        if let Some(tok) = next_scripted() {
            let t = tok.trim();
            return match game {
                GameKind::Ttt => parse_ttt_token(t).unwrap_or(Input::Resign),
                GameKind::Hangman => parse_hangman_token(t).unwrap_or(Input::Resign),
            };
        }
        return Input::Resign;
    }

    let prompt = match game {
        GameKind::Ttt => "Your move (0-8), or q to resign: ",
        GameKind::Hangman => "Your letter, or q to resign: ",
    };
    loop {
        eprint!("{prompt}");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() {
            return Input::Resign;
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let parsed = match game {
            GameKind::Ttt => parse_ttt_token(t),
            GameKind::Hangman => parse_hangman_token(t),
        };
        if let Some(inp) = parsed {
            return inp;
        }
        eprintln!("invalid input `{t}`");
    }
}

fn input_to_msg(inp: Input) -> ClientMsg {
    match inp {
        Input::Place(index) => ClientMsg::Place { index },
        Input::Guess(letter) => ClientMsg::Guess { letter },
        Input::Resign => ClientMsg::Resign,
    }
}

fn persist_match_ended(msg: &ServerMsg) {
    if let ServerMsg::MatchEnded {
        host_id,
        guest_id,
        balance_host,
        balance_guest,
        winner_id,
        ..
    } = msg
    {
        let mut updates = vec![(host_id.clone(), *balance_host)];
        if let Some(g) = guest_id {
            updates.push((g.clone(), *balance_guest));
        }
        if let Err(e) = merge_balances(&updates) {
            eprintln!("warn: ledger merge failed: {e}");
        } else {
            println!(
                "Ledger updated at {} (winner={winner_id:?})",
                ledger_path().display()
            );
        }
    }
}

fn broadcast(
    writer: &mut TcpStream,
    ev: &RoomEvent,
    my_role: &mut Option<Role>,
    last_ended: &mut Option<ServerMsg>,
) -> Result<(), Box<dyn std::error::Error>> {
    for m in &ev.to_guest {
        write_msg(writer, m)?;
    }
    for m in &ev.to_host {
        if apply_server_msg(m, my_role) {
            *last_ended = Some(m.clone());
        }
    }
    Ok(())
}

fn open_host_room(opts: &HostOpts, room_code: &str) -> Result<HostRoom, RoomError> {
    let ledger = load_ledger();
    match opts.game {
        GameKind::Ttt => HostRoom::ttt(room_code, &opts.name, opts.stake, ledger),
        GameKind::Hangman => {
            if let Some(word) = &opts.word {
                HostRoom::hangman(
                    room_code,
                    &opts.name,
                    opts.stake,
                    ledger,
                    word,
                    DEFAULT_MAX_ATTEMPTS,
                )
            } else {
                let seed = opts.seed.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(1)
                });
                HostRoom::hangman_from_seed(
                    room_code,
                    &opts.name,
                    opts.stake,
                    ledger,
                    seed,
                    ENGLISH_MOVIES,
                    DEFAULT_MAX_ATTEMPTS,
                )
            }
        }
    }
}

fn join_error_hint(e: RoomError, code: &str, room_code: &str, local_port: u16) -> String {
    match e {
        RoomError::BadCode => format!(
            "room code mismatch: guest sent `{code}`, host room is `{room_code}`. \
Reconnect with the exact code from the host's BET_READY line, e.g. \
bet join {room_code}@127.0.0.1:{local_port}"
        ),
        RoomError::GameMismatch { room, guest } => format!(
            "game mismatch: this room is {room}, guest asked for {guest}. \
Rejoin with --game {room}"
        ),
        RoomError::StakeMismatch { need, got } => {
            format!("stake mismatch: this room needs {need}, guest offered {got}")
        }
        other => other.to_string(),
    }
}

pub fn run_host(opts: HostOpts) -> Result<(), Box<dyn std::error::Error>> {
    if opts.stake <= 0 {
        return Err("stake must be > 0".into());
    }
    let room_code = match opts.code {
        Some(ref c) => normalize_code(c)?,
        None => gen_room_code(),
    };
    let mut room = open_host_room(&opts, &room_code)?;

    let bind = format!("{}:{}", opts.bind, opts.port);
    let listener = TcpListener::bind(&bind)?;
    let local_port = listener.local_addr()?.port();
    let game = opts.game;

    println!("BET multiplayer host ({game}, virtual points)");
    println!("  you:     {} (X)", opts.name);
    println!(
        "  stake:   {} each (pot will be {})",
        opts.stake,
        opts.stake * 2
    );
    println!("  balance: {}", room.ledger().balance(&opts.name));
    println!("  room:    {room_code}");
    println!("  listen:  {bind} (port {local_port})");
    println!("  config:  {}", ledger_path().display());
    println!("  proto:   {PROTOCOL_VERSION}");
    println!(
        "BET_READY room={room_code} port={local_port} stake={} proto={PROTOCOL_VERSION} game={game}",
        opts.stake
    );
    println!();
    println!("Guest runs:");
    println!(
        "  bet join {room_code}@127.0.0.1:{local_port} --stake {} --game {game}",
        opts.stake
    );
    println!("Waiting for guest…");

    let (stream, peer) = listener.accept()?;
    println!("Connected: {peer}");
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    stream.set_nodelay(true)?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let hello = decode_client_line(&line)?;
    let code = match &hello {
        ClientMsg::Hello { room_code: c, .. } => c.clone(),
        _ => {
            write_msg(
                &mut writer,
                &ServerMsg::Error {
                    message: "expected hello".into(),
                },
            )?;
            return Err("expected hello".into());
        }
    };

    let ev = match room.handle(false, hello) {
        Ok(ev) => ev,
        Err(e) => {
            let hint = join_error_hint(e, &code, &room_code, local_port);
            write_msg(
                &mut writer,
                &ServerMsg::Error {
                    message: hint.clone(),
                },
            )?;
            return Err(format!("join failed: {hint}").into());
        }
    };

    let mut my_role = Some(Role::X);
    let mut last_ended: Option<ServerMsg> = None;
    broadcast(&mut writer, &ev, &mut my_role, &mut last_ended)?;

    let (tx, rx) = mpsc::channel::<Result<ClientMsg, String>>();
    thread::spawn(move || {
        let mut reader = reader;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    let _ = tx.send(Err("guest disconnected".into()));
                    break;
                }
                Ok(_) => match decode_client_line(&line) {
                    Ok(msg) => {
                        if tx.send(Ok(msg)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("bad json: {e}")));
                        break;
                    }
                },
                Err(e) => {
                    let _ = tx.send(Err(format!("read: {e}")));
                    break;
                }
            }
        }
    });

    while room.phase() == Phase::Playing {
        if room.host_turn() {
            let msg = input_to_msg(read_input(game));
            match room.handle(true, msg) {
                Ok(ev) => broadcast(&mut writer, &ev, &mut my_role, &mut last_ended)?,
                Err(e) => eprintln!("illegal: {e:?}"),
            }
        } else {
            println!("Waiting for guest…");
            match rx.recv_timeout(Duration::from_secs(300)) {
                Ok(Ok(msg)) => match room.handle(false, msg) {
                    Ok(ev) => broadcast(&mut writer, &ev, &mut my_role, &mut last_ended)?,
                    Err(e) => {
                        write_msg(
                            &mut writer,
                            &ServerMsg::Error {
                                message: format!("{e:?}"),
                            },
                        )?;
                    }
                },
                Ok(Err(e)) => {
                    if room.phase() == Phase::Playing {
                        eprintln!("guest lost: {e} — treating as forfeit");
                        if let Ok(ev) = room.disconnect(false) {
                            let _ = broadcast(&mut writer, &ev, &mut my_role, &mut last_ended);
                        }
                    } else {
                        return Err(e.into());
                    }
                }
                Err(_) => return Err("timeout waiting for guest move".into()),
            }
        }
    }

    save_ledger(room.ledger())?;
    if let Some(ended) = last_ended {
        persist_match_ended(&ended);
    }
    println!("Ledger saved to {}", ledger_path().display());
    Ok(())
}

pub fn run_join(opts: JoinOpts) -> Result<(), Box<dyn std::error::Error>> {
    if opts.stake <= 0 {
        return Err("stake must be > 0".into());
    }
    let room_code = normalize_code(&opts.room_code)?;

    let mut ledger = load_ledger();
    ledger.ensure_player(opts.name.clone());
    let _ = save_ledger(&ledger);

    println!(
        "Joining room {room_code} at {} as {} (game {}, stake {}, proto {PROTOCOL_VERSION})…",
        opts.addr, opts.name, opts.game, opts.stake
    );
    println!("  config: {}", ledger_path().display());

    let stream = TcpStream::connect(&opts.addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    stream.set_nodelay(true)?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    write_msg(
        &mut writer,
        &ClientMsg::Hello {
            player_id: opts.name.clone(),
            room_code,
            stake: opts.stake,
            proto: PROTOCOL_VERSION,
            game: opts.game,
        },
    )?;

    let mut my_role: Option<Role> = None;
    let mut game = opts.game;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            eprintln!("host closed connection");
            break;
        }
        let msg = decode_server_line(&line)?;
        if let ServerMsg::Welcome { game: g, .. } = &msg {
            game = *g;
        }
        let is_end = apply_server_msg(&msg, &mut my_role);
        if is_end {
            persist_match_ended(&msg);
            break;
        }

        let your_turn = match &msg {
            ServerMsg::State { your_turn, .. } | ServerMsg::HangmanState { your_turn, .. } => {
                *your_turn
            }
            _ => false,
        };
        if your_turn && my_role == Some(Role::O) {
            write_msg(&mut writer, &input_to_msg(read_input(game)))?;
        }
        if let ServerMsg::Error { message } = &msg {
            return Err(message.clone().into());
        }
    }

    Ok(())
}

fn write_msg(
    w: &mut impl Write,
    msg: &impl serde::Serialize,
) -> Result<(), Box<dyn std::error::Error>> {
    let line = encode_line(msg)?;
    w.write_all(line.as_bytes())?;
    w.flush()?;
    Ok(())
}

pub fn print_balances() {
    let led = load_ledger();
    println!("Ledger: {}", ledger_path().display());
    println!("default_grant: {}", led.default_grant());
    if led.balances().is_empty() {
        println!("(empty — play a match first)");
        let id = default_player_id();
        println!("hint: id=`{id}` ($BET_PLAYER or $USER); BET_CONFIG_DIR overrides path");
        return;
    }
    let mut rows: Vec<_> = led.balances().iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (k, v) in rows {
        println!("  {k}: {v}");
    }
}

/// Parse `CODE` or `CODE@host:port` into (code, optional addr).
pub fn parse_join_target(raw: &str) -> (String, Option<String>) {
    if let Some((code, addr)) = raw.split_once('@') {
        (code.to_string(), Some(addr.to_string()))
    } else {
        (raw.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_join_target_splits_at() {
        let (code, addr) = parse_join_target("AB12@127.0.0.1:9");
        assert_eq!(code, "AB12");
        assert_eq!(addr.as_deref(), Some("127.0.0.1:9"));
        let (code, addr) = parse_join_target("AB12");
        assert_eq!(code, "AB12");
        assert_eq!(addr, None);
    }

    #[test]
    fn hangman_token_is_single_letter() {
        assert!(matches!(parse_hangman_token("B"), Some(Input::Guess('B'))));
        assert!(matches!(parse_hangman_token("q"), Some(Input::Resign)));
        assert!(parse_hangman_token("BE").is_none());
        assert!(parse_hangman_token("1").is_none());
    }

    #[test]
    fn ttt_token_is_digit() {
        assert!(matches!(parse_ttt_token("4"), Some(Input::Place(4))));
        assert!(parse_ttt_token("9").is_none());
        assert!(matches!(parse_ttt_token("resign"), Some(Input::Resign)));
    }
}
