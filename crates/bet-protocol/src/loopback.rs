//! In-process two-player transport for tests (no sockets).

use crate::msg::{ClientMsg, ServerMsg};
use crate::room::{RoomError, TttRoom};

/// Drive a full scripted match on a room; returns final host/guest balances.
pub fn play_scripted(
    room: &mut TttRoom,
    guest_id: &str,
    stake: i64,
    moves: &[(bool, u8)],
) -> Result<(i64, i64, Vec<ServerMsg>, Vec<ServerMsg>), RoomError> {
    let mut host_log = Vec::new();
    let mut guest_log = Vec::new();

    let join = ClientMsg::Hello {
        player_id: guest_id.into(),
        room_code: room.room_code().to_string(),
        stake,
        proto: crate::msg::PROTOCOL_VERSION,
        game: crate::msg::GameKind::Ttt,
    };
    let ev = room.handle(false, join)?;
    host_log.extend(ev.to_host);
    guest_log.extend(ev.to_guest);

    for &(from_host, index) in moves {
        let ev = room.handle(from_host, ClientMsg::Place { index })?;
        host_log.extend(ev.to_host);
        guest_log.extend(ev.to_guest);
    }

    let hb = room.ledger().balance(room.host_id());
    let gb = room
        .guest_id()
        .map(|g| room.ledger().balance(g))
        .unwrap_or(0);
    Ok((hb, gb, host_log, guest_log))
}

/// Drive a scripted hangman match (letters, not board indices).
pub fn play_scripted_hangman(
    room: &mut crate::hangman_room::HangmanRoom,
    guest_id: &str,
    stake: i64,
    guesses: &[(bool, char)],
) -> Result<(i64, i64, Vec<ServerMsg>, Vec<ServerMsg>), RoomError> {
    let mut host_log = Vec::new();
    let mut guest_log = Vec::new();

    let join = ClientMsg::Hello {
        player_id: guest_id.into(),
        room_code: room.room_code().to_string(),
        stake,
        proto: crate::msg::PROTOCOL_VERSION,
        game: crate::msg::GameKind::Hangman,
    };
    let ev = room.handle(false, join)?;
    host_log.extend(ev.to_host);
    guest_log.extend(ev.to_guest);

    for &(from_host, letter) in guesses {
        let ev = room.handle(from_host, ClientMsg::Guess { letter })?;
        host_log.extend(ev.to_host);
        guest_log.extend(ev.to_guest);
    }

    let hb = room.ledger().balance(room.host_id());
    let gb = room
        .guest_id()
        .map(|g| room.ledger().balance(g))
        .unwrap_or(0);
    Ok((hb, gb, host_log, guest_log))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hangman_room::HangmanRoom;
    use crate::msg::ServerMsg;
    use crate::room::TttRoom;
    use bet_core::ledger::Ledger;

    #[test]
    fn loopback_ttt_settles() {
        let mut led = Ledger::new(100);
        led.ensure_player("h");
        led.ensure_player("g");
        let mut room = TttRoom::new("ZZ99", "h", 10, led).unwrap();
        // Host wins: X 0,1,2
        let moves = [
            (true, 0),
            (false, 3),
            (true, 1),
            (false, 4),
            (true, 2),
        ];
        let (hb, gb, hlog, _) = play_scripted(&mut room, "g", 10, &moves).unwrap();
        assert_eq!(hb, 110);
        assert_eq!(gb, 90);
        assert!(hlog.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(crate::msg::Role::X),
                ..
            }
        )));
    }

    #[test]
    fn loopback_hangman_host_solves() {
        let mut led = Ledger::new(100);
        led.ensure_player("h");
        led.ensure_player("g");
        let mut room = HangmanRoom::new("HM01", "h", 10, led, "BET", 6).unwrap();
        let (hb, gb, hlog, _) =
            play_scripted_hangman(&mut room, "g", 10, &[(true, 'B'), (true, 'E'), (true, 'T')])
                .unwrap();
        assert_eq!(hb, 110);
        assert_eq!(gb, 90);
        assert!(hlog.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(crate::msg::Role::X),
                word: Some(w),
                ..
            } if w == "BET"
        )));
    }

    #[test]
    fn old_ttt_hello_without_game_field_still_joins() {
        // proto v1 additive: missing `game` must deserialize as Ttt.
        let raw = r#"{"type":"hello","player_id":"g","room_code":"ZZ99","stake":10,"proto":1}"#;
        let msg: ClientMsg = serde_json::from_str(raw).unwrap();
        match msg {
            ClientMsg::Hello { game, proto, .. } => {
                assert_eq!(game, crate::msg::GameKind::Ttt);
                assert_eq!(proto, crate::msg::PROTOCOL_VERSION);
            }
            other => panic!("expected hello, got {other:?}"),
        }
    }
}
