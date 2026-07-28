//! Multiplayer host/join over TCP (newline-delimited JSON).

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use bet_core::rng::XorShift64;
use bet_protocol::msg::{
    decode_client_line, decode_server_line, encode_line, ClientMsg, Role, ServerMsg,
};
use bet_protocol::room::{Phase, TttRoom};

use crate::ledger_store::{default_player_id, load_ledger, merge_player_balance, save_ledger};

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
        ServerMsg::Welcome { role, stake, room_code, match_id, player_id } => {
            *my_role = Some(*role);
            println!(
                "Welcome {player_id} as {:?} | stake={stake} | room={room_code} | match={match_id}",
                role
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
            balance_host,
            balance_guest,
        } => {
            println!("=== MATCH ENDED ===");
            println!("winner={winner:?} winner_id={winner_id:?} pot={pot}");
            println!("balances: host={balance_host} guest={balance_guest}");
            true
        }
        ServerMsg::Error { message } => {
            eprintln!("error: {message}");
            false
        }
        ServerMsg::Pong => false,
    }
}

fn read_move_stdin() -> Option<u8> {
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

/// Host authoritative room; host plays as X on stdin; guest via TCP.
pub fn run_host(opts: HostOpts) -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = load_ledger();
    let room_code = gen_room_code();
    let mut room = TttRoom::new(&room_code, &opts.name, opts.stake, ledger.clone())?;

    let bind = format!("{}:{}", opts.bind, opts.port);
    let listener = TcpListener::bind(&bind)?;
    listener.set_nonblocking(false)?;

    println!("BET multiplayer host (tic-tac-toe, virtual points)");
    println!("  you:     {} (X)", opts.name);
    println!("  stake:   {} each (pot will be {})", opts.stake, opts.stake * 2);
    println!("  balance: {}", room.ledger.balance(&opts.name));
    println!("  room:    {room_code}");
    println!("  listen:  {bind}");
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

    // Expect Hello
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let hello = decode_client_line(&line)?;
    let ClientMsg::Hello {
        player_id,
        room_code: code,
        stake,
    } = hello
    else {
        write_msg(&mut writer, &ServerMsg::Error {
            message: "expected hello".into(),
        })?;
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

    // Channel: network thread → main
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
            match read_move_stdin() {
                Some(idx) => {
                    match room.handle(true, ClientMsg::Place { index: idx }) {
                        Ok(ev) => {
                            for m in &ev.to_guest {
                                write_msg(&mut writer, m)?;
                            }
                            for m in &ev.to_host {
                                if apply_server_msg(m, &mut my_role) {
                                    // ended
                                }
                            }
                        }
                        Err(e) => eprintln!("illegal: {e:?}"),
                    }
                }
                None => {
                    let ev = room.handle(true, ClientMsg::Resign)?;
                    for m in &ev.to_guest {
                        write_msg(&mut writer, m)?;
                    }
                    for m in &ev.to_host {
                        apply_server_msg(m, &mut my_role);
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
                            apply_server_msg(m, &mut my_role);
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

    // Persist host's view of ledger (both balances updated on host).
    ledger = room.ledger;
    save_ledger(&ledger)?;
    println!("Ledger saved to {}", crate::ledger_store::ledger_path().display());
    Ok(())
}

pub fn run_join(opts: JoinOpts) -> Result<(), Box<dyn std::error::Error>> {
    // Guest ledger is local for display only; host is authority for settlement.
    // After match, guest updates local balance from MatchEnded if same player ids.
    let mut ledger = load_ledger();
    ledger.ensure_player(opts.name.clone());

    println!(
        "Joining room {} at {} as {} (stake {})…",
        opts.room_code, opts.addr, opts.name, opts.stake
    );
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
            if let ServerMsg::MatchEnded {
                balance_guest,
                winner_id,
                ..
            } = &msg
            {
                if my_role == Some(Role::O) {
                    let _ = merge_player_balance(&opts.name, *balance_guest);
                    println!(
                        "Local ledger updated (guest balance={balance_guest}). winner={winner_id:?}"
                    );
                }
            }
            break;
        }

        if let ServerMsg::State { your_turn, .. } = &msg
            && *your_turn
            && my_role == Some(Role::O)
        {
            match read_move_stdin() {
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

fn write_msg(w: &mut impl Write, msg: &impl serde::Serialize) -> Result<(), Box<dyn std::error::Error>> {
    let line = encode_line(msg)?;
    w.write_all(line.as_bytes())?;
    w.flush()?;
    Ok(())
}

pub fn print_balances() {
    let led = load_ledger();
    println!("Ledger: {}", crate::ledger_store::ledger_path().display());
    println!("default_grant: {}", led.default_grant());
    if led.balances().is_empty() {
        println!("(empty — play a match or balances appear after ensure)");
        let id = default_player_id();
        println!("hint: your id would be `{id}` (BET_PLAYER or $USER)");
        return;
    }
    for (k, v) in led.balances() {
        println!("  {k}: {v}");
    }
}
