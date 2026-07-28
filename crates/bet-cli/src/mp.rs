//! Multiplayer host/join over TCP (newline-delimited JSON).
//!
//! Scripted (non-interactive) moves for CI: `BET_MOVES=0,3,1,4,2`

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use bet_core::rng::XorShift64;
use bet_protocol::msg::{
    decode_client_line, decode_server_line, encode_line, ClientMsg, Role, ServerMsg,
};
use bet_protocol::room::{Phase, TttRoom};

use crate::ledger_store::{
    default_player_id, ledger_path, load_ledger, merge_balances, save_ledger,
};

pub const DEFAULT_PORT: u16 = 7733;

pub struct HostOpts {
    pub stake: i64,
    pub port: u16,
    pub name: String,
    pub bind: String,
}

pub struct JoinOpts {
    pub stake: i64,
    pub addr: String,
    pub room_code: String,
    pub name: String,
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

/// Shared scripted move queue from `BET_MOVES` (comma-separated 0-8 or q).
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
            pretty(board[i]),
            pretty(board[i + 1]),
            pretty(board[i + 2])
        );
        if row < 2 {
            println!(" ---+---+---");
        }
    }
    println!();
}

fn pretty(c: char) -> char {
    if c == '.' {
        ' '
    } else {
        c
    }
}

fn apply_server_msg(msg: &ServerMsg, my_role: &mut Option<Role>) -> bool {
    match msg {
        ServerMsg::Welcome {
            role,
            stake,
            room_code,
            match_id,
            player_id,
        } => {
            *my_role = Some(*role);
            println!(
                "Welcome {player_id} as {role:?} | stake={stake} | room={room_code} | match={match_id}"
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
        } => {
            println!("=== MATCH ENDED ===");
            println!("winner={winner:?} winner_id={winner_id:?} pot={pot}");
            println!("host={host_id}={balance_host} guest={guest_id:?}={balance_guest}");
            true
        }
        ServerMsg::Error { message } => {
            eprintln!("error: {message}");
            false
        }
        ServerMsg::Pong => false,
    }
}

/// Returns Some(index) for a place, None for resign / EOF.
fn read_move() -> Option<u8> {
    // Prefer scripted moves (CI / automation).
    if let Ok(mut q) = move_queue().lock() {
        if let Some(tok) = q.pop_front() {
            let t = tok.trim();
            if t.eq_ignore_ascii_case("q") || t.eq_ignore_ascii_case("resign") {
                return None;
            }
            return t.parse::<u8>().ok().filter(|&n| n < 9);
        }
    }

    eprint!("Your move (0-8), or q to resign: ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return None;
    }
    let t = line.trim();
    if t.eq_ignore_ascii_case("q") || t.eq_ignore_ascii_case("resign") {
        return None;
    }
    t.parse::<u8>().ok().filter(|&n| n < 9)
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

/// Host authoritative room; host plays as X; guest via TCP.
pub fn run_host(opts: HostOpts) -> Result<(), Box<dyn std::error::Error>> {
    let ledger = load_ledger();
    let room_code = gen_room_code();
    let mut room = TttRoom::new(&room_code, &opts.name, opts.stake, ledger)?;

    let bind = format!("{}:{}", opts.bind, opts.port);
    let listener = TcpListener::bind(&bind)?;

    println!("BET multiplayer host (tic-tac-toe, virtual points)");
    println!("  you:     {} (X)", opts.name);
    println!(
        "  stake:   {} each (pot will be {})",
        opts.stake,
        opts.stake * 2
    );
    println!("  balance: {}", room.ledger.balance(&opts.name));
    println!("  room:    {room_code}");
    println!("  listen:  {bind}");
    println!("  config:  {}", ledger_path().display());
    println!();
    println!("Guest runs:");
    println!(
        "  bet join {room_code} --stake {} --addr 127.0.0.1:{}",
        opts.stake, opts.port
    );
    println!("Waiting for guest…");

    let (stream, peer) = listener.accept()?;
    println!("Connected: {peer}");
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    let mut line = String::new();
    reader.read_line(&mut line)?;
    let hello = decode_client_line(&line)?;
    let ClientMsg::Hello {
        player_id,
        room_code: code,
        stake,
    } = hello
    else {
        write_msg(
            &mut writer,
            &ServerMsg::Error {
                message: "expected hello".into(),
            },
        )?;
        return Err("expected hello".into());
    };

    let ev = match room.handle(
        false,
        ClientMsg::Hello {
            player_id: player_id.clone(),
            room_code: code,
            stake,
        },
    ) {
        Ok(ev) => ev,
        Err(e) => {
            write_msg(
                &mut writer,
                &ServerMsg::Error {
                    message: format!("{e:?}"),
                },
            )?;
            return Err(format!("join failed: {e:?}").into());
        }
    };

    for m in &ev.to_guest {
        write_msg(&mut writer, m)?;
    }
    let mut my_role = Some(Role::X);
    for m in &ev.to_host {
        apply_server_msg(m, &mut my_role);
    }

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

    let mut last_ended: Option<ServerMsg> = None;

    while room.phase == Phase::Playing {
        let your_turn = room.game.current == bet_core::tictactoe::Player::X;
        if your_turn {
            match read_move() {
                Some(idx) => match room.handle(true, ClientMsg::Place { index: idx }) {
                    Ok(ev) => {
                        for m in &ev.to_guest {
                            write_msg(&mut writer, m)?;
                        }
                        for m in &ev.to_host {
                            if apply_server_msg(m, &mut my_role) {
                                last_ended = Some(m.clone());
                            }
                        }
                    }
                    Err(e) => eprintln!("illegal: {e:?}"),
                },
                None => {
                    let ev = room.handle(true, ClientMsg::Resign)?;
                    for m in &ev.to_guest {
                        write_msg(&mut writer, m)?;
                    }
                    for m in &ev.to_host {
                        if apply_server_msg(m, &mut my_role) {
                            last_ended = Some(m.clone());
                        }
                    }
                }
            }
        } else {
            println!("Waiting for guest…");
            match rx.recv_timeout(Duration::from_secs(300)) {
                Ok(Ok(msg)) => match room.handle(false, msg) {
                    Ok(ev) => {
                        for m in &ev.to_guest {
                            write_msg(&mut writer, m)?;
                        }
                        for m in &ev.to_host {
                            if apply_server_msg(m, &mut my_role) {
                                last_ended = Some(m.clone());
                            }
                        }
                    }
                    Err(e) => {
                        write_msg(
                            &mut writer,
                            &ServerMsg::Error {
                                message: format!("{e:?}"),
                            },
                        )?;
                    }
                },
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => return Err("timeout waiting for guest move".into()),
            }
        }
    }

    // Host is authority: write full ledger, then also merge MatchEnded ids.
    save_ledger(&room.ledger)?;
    if let Some(ended) = last_ended {
        persist_match_ended(&ended);
    }
    println!("Ledger saved to {}", ledger_path().display());
    Ok(())
}

pub fn run_join(opts: JoinOpts) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure guest row exists locally before match (display / offline).
    let mut ledger = load_ledger();
    ledger.ensure_player(opts.name.clone());
    let _ = save_ledger(&ledger);

    println!(
        "Joining room {} at {} as {} (stake {})…",
        opts.room_code, opts.addr, opts.name, opts.stake
    );
    println!("  config: {}", ledger_path().display());

    let stream = TcpStream::connect(&opts.addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(300)))?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    write_msg(
        &mut writer,
        &ClientMsg::Hello {
            player_id: opts.name.clone(),
            room_code: opts.room_code.clone(),
            stake: opts.stake,
        },
    )?;

    let mut my_role: Option<Role> = None;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let msg = decode_server_line(&line)?;
        let is_end = apply_server_msg(&msg, &mut my_role);
        if is_end {
            persist_match_ended(&msg);
            break;
        }

        if let ServerMsg::State { your_turn, .. } = &msg
            && *your_turn
            && my_role == Some(Role::O)
        {
            match read_move() {
                Some(idx) => write_msg(&mut writer, &ClientMsg::Place { index: idx })?,
                None => write_msg(&mut writer, &ClientMsg::Resign)?,
            }
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
        println!("(empty — play a match or balances appear after ensure)");
        let id = default_player_id();
        println!("hint: your id would be `{id}` ($BET_PLAYER or $USER)");
        println!("hint: set BET_CONFIG_DIR to override ledger location");
        return;
    }
    for (k, v) in led.balances() {
        println!("  {k}: {v}");
    }
}
