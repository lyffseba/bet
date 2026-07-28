//! Multiplayer host/join over TCP (newline-delimited JSON).
//!
//! - Scripted moves: `BET_MOVES=0,3,1,4,2`
//! - Machine line: `BET_READY room=CODE port=P stake=N proto=1`
//! - Empty cells show indices 0–8 for clearer play

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
    PROTOCOL_VERSION,
};
use bet_protocol::room::{normalize_code, Phase, TttRoom};

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

fn apply_server_msg(msg: &ServerMsg, my_role: &mut Option<Role>) -> bool {
    match msg {
        ServerMsg::Welcome {
            role,
            stake,
            room_code,
            match_id,
            player_id,
            proto,
        } => {
            *my_role = Some(*role);
            println!(
                "Welcome {player_id} as {role:?} | stake={stake} | room={room_code} | match={match_id} | proto={proto}"
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

fn read_move() -> Option<u8> {
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

fn broadcast(
    writer: &mut TcpStream,
    ev: &bet_protocol::room::RoomEvent,
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

pub fn run_host(opts: HostOpts) -> Result<(), Box<dyn std::error::Error>> {
    if opts.stake <= 0 {
        return Err("stake must be > 0".into());
    }
    let ledger = load_ledger();
    let room_code = match opts.code {
        Some(c) => normalize_code(c)?,
        None => gen_room_code(),
    };
    let mut room = TttRoom::new(&room_code, &opts.name, opts.stake, ledger)?;

    let bind = format!("{}:{}", opts.bind, opts.port);
    let listener = TcpListener::bind(&bind)?;
    let local_port = listener.local_addr()?.port();

    println!("BET multiplayer host (tic-tac-toe, virtual points)");
    println!("  you:     {} (X)", opts.name);
    println!(
        "  stake:   {} each (pot will be {})",
        opts.stake,
        opts.stake * 2
    );
    println!("  balance: {}", room.ledger.balance(&opts.name));
    println!("  room:    {room_code}");
    println!("  listen:  {bind} (port {local_port})");
    println!("  config:  {}", ledger_path().display());
    println!("  proto:   {PROTOCOL_VERSION}");
    // Machine-parseable ready line for e2e / tooling.
    println!(
        "BET_READY room={room_code} port={local_port} stake={} proto={PROTOCOL_VERSION}",
        opts.stake
    );
    println!();
    println!("Guest runs:");
    println!(
        "  bet join {room_code}@127.0.0.1:{local_port} --stake {}",
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
    let ClientMsg::Hello {
        player_id,
        room_code: code,
        stake,
        proto,
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
            proto,
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

    while room.phase == Phase::Playing {
        let your_turn = room.game.current == bet_core::tictactoe::Player::X;
        if your_turn {
            match read_move() {
                Some(idx) => match room.handle(true, ClientMsg::Place { index: idx }) {
                    Ok(ev) => broadcast(&mut writer, &ev, &mut my_role, &mut last_ended)?,
                    Err(e) => eprintln!("illegal: {e:?}"),
                },
                None => {
                    let ev = room.handle(true, ClientMsg::Resign)?;
                    broadcast(&mut writer, &ev, &mut my_role, &mut last_ended)?;
                }
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
                    // Disconnect / protocol error mid-match → guest forfeits.
                    if room.phase == Phase::Playing {
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

    save_ledger(&room.ledger)?;
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
        "Joining room {room_code} at {} as {} (stake {}, proto {PROTOCOL_VERSION})…",
        opts.addr, opts.name, opts.stake
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
        },
    )?;

    let mut my_role: Option<Role> = None;

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            eprintln!("host closed connection");
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
