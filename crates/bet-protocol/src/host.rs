//! Game-kind dispatch for a host-authoritative room.
//!
//! CLI / transports own sockets. This enum owns “which game is this table”.

use bet_core::ledger::Ledger;

use crate::hangman_room::{HangmanRoom, DEFAULT_MAX_ATTEMPTS};
use crate::msg::{ClientMsg, GameKind};
use crate::room::{Phase, RoomError, RoomEvent, TttRoom};

/// One live match: either tic-tac-toe or hangman.
pub enum HostRoom {
    Ttt(TttRoom),
    Hangman(HangmanRoom),
}

impl HostRoom {
    pub fn ttt(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
    ) -> Result<Self, RoomError> {
        Ok(Self::Ttt(TttRoom::new(room_code, host_id, stake, ledger)?))
    }

    pub fn hangman(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
        word: impl Into<String>,
        max_attempts: usize,
    ) -> Result<Self, RoomError> {
        Ok(Self::Hangman(HangmanRoom::new(
            room_code,
            host_id,
            stake,
            ledger,
            word,
            max_attempts,
        )?))
    }

    pub fn hangman_from_seed(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
        seed: u64,
        words: &[&str],
        max_attempts: usize,
    ) -> Result<Self, RoomError> {
        Ok(Self::Hangman(HangmanRoom::from_seed(
            room_code,
            host_id,
            stake,
            ledger,
            seed,
            words,
            max_attempts.max(DEFAULT_MAX_ATTEMPTS),
        )?))
    }

    pub fn kind(&self) -> GameKind {
        match self {
            Self::Ttt(_) => GameKind::Ttt,
            Self::Hangman(_) => GameKind::Hangman,
        }
    }

    pub fn handle(&mut self, from_host: bool, msg: ClientMsg) -> Result<RoomEvent, RoomError> {
        match self {
            Self::Ttt(r) => r.handle(from_host, msg),
            Self::Hangman(r) => r.handle(from_host, msg),
        }
    }

    pub fn disconnect(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        match self {
            Self::Ttt(r) => r.disconnect(from_host),
            Self::Hangman(r) => r.disconnect(from_host),
        }
    }

    pub fn phase(&self) -> Phase {
        match self {
            Self::Ttt(r) => r.phase(),
            Self::Hangman(r) => r.phase(),
        }
    }

    pub fn host_turn(&self) -> bool {
        match self {
            Self::Ttt(r) => r.host_turn(),
            Self::Hangman(r) => r.host_turn(),
        }
    }

    pub fn ledger(&self) -> &Ledger {
        match self {
            Self::Ttt(r) => r.ledger(),
            Self::Hangman(r) => r.ledger(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{Role, ServerMsg, PROTOCOL_VERSION};
    use bet_core::ledger::Ledger;

    #[test]
    fn kind_follows_constructor() {
        let t = HostRoom::ttt("ABCD", "h", 10, Ledger::new(100)).unwrap();
        assert_eq!(t.kind(), GameKind::Ttt);
        let h = HostRoom::hangman("ABCD", "h", 10, Ledger::new(100), "BET", 6).unwrap();
        assert_eq!(h.kind(), GameKind::Hangman);
    }

    #[test]
    fn hangman_dispatch_rejects_place() {
        let mut r = HostRoom::hangman("ABCD", "h", 10, Ledger::new(100), "BET", 6).unwrap();
        let hello = ClientMsg::Hello {
            player_id: "g".into(),
            room_code: "ABCD".into(),
            stake: 10,
            proto: PROTOCOL_VERSION,
            game: GameKind::Hangman,
        };
        r.handle(false, hello).unwrap();
        assert!(matches!(
            r.handle(true, ClientMsg::Place { index: 0 }),
            Err(RoomError::WrongAction)
        ));
    }

    #[test]
    fn ttt_dispatch_host_wins() {
        let mut r = HostRoom::ttt("ABCD", "h", 10, Ledger::new(100)).unwrap();
        r.handle(
            false,
            ClientMsg::Hello {
                player_id: "g".into(),
                room_code: "ABCD".into(),
                stake: 10,
                proto: PROTOCOL_VERSION,
                game: GameKind::Ttt,
            },
        )
        .unwrap();
        r.handle(true, ClientMsg::Place { index: 0 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 3 }).unwrap();
        r.handle(true, ClientMsg::Place { index: 1 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 4 }).unwrap();
        let ev = r.handle(true, ClientMsg::Place { index: 2 }).unwrap();
        assert_eq!(r.phase(), Phase::Ended);
        assert!(ev.to_host.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::X),
                ..
            }
        )));
    }
}
