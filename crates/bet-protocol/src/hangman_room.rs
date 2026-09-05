//! Host-authoritative hangman room with virtual stakes.
//!
//! Host is Role::X and takes the first guess. The secret word is never put
//! on the wire until `MatchEnded`. A miss flips the turn; a hit keeps it.
//! Illegal guesses (`AlreadyGuessed`, `NotLetter`) are rejected and do not
//! consume an attempt or flip the turn.

use bet_core::hangman::{GuessError, Hangman};
use bet_core::ledger::Ledger;

use crate::msg::{ClientMsg, GameKind, Role, ServerMsg};
use crate::room::{Phase, RoomError, RoomEvent};
use crate::table::Table;

/// Default NATO-style bank used by tests / `--seed` when the CLI does not
/// supply its larger movie list.
pub const DEFAULT_WORDS: &[&str] = &[
    "ALPHA", "BRAVO", "CHARLIE", "DELTA", "ECHO", "FOXTROT", "GOLF", "HOTEL",
];

pub const DEFAULT_MAX_ATTEMPTS: usize = 6;

/// One host + one guest, hangman, virtual pot.
pub struct HangmanRoom {
    pub table: Table,
    pub game: Hangman,
    pub current: Role,
    last_guess: Option<char>,
    last_hit: Option<bool>,
}

impl HangmanRoom {
    pub fn new(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
        word: impl Into<String>,
        max_attempts: usize,
    ) -> Result<Self, RoomError> {
        let word = word.into();
        if word.chars().filter(|c| c.is_alphabetic()).count() == 0 {
            return Err(RoomError::EmptyWord);
        }
        Ok(Self {
            table: Table::open(room_code, host_id, stake, ledger)?,
            game: Hangman::new(&word, max_attempts.max(1)),
            current: Role::X,
            last_guess: None,
            last_hit: None,
        })
    }

    /// Pick a word from `words` with a seeded PRNG (deterministic).
    pub fn from_seed(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
        seed: u64,
        words: &[&str],
        max_attempts: usize,
    ) -> Result<Self, RoomError> {
        if words.is_empty() {
            return Err(RoomError::EmptyWordList);
        }
        let picked = Hangman::from_seed(seed, words, max_attempts.max(1));
        let word = picked.word().to_string();
        Self::new(room_code, host_id, stake, ledger, word, max_attempts)
    }

    pub fn room_code(&self) -> &str {
        self.table.room_code()
    }
    pub fn match_id(&self) -> &str {
        self.table.match_id()
    }
    pub fn host_id(&self) -> &str {
        self.table.host_id()
    }
    pub fn guest_id(&self) -> Option<&str> {
        self.table.guest_id()
    }
    pub fn phase(&self) -> Phase {
        self.table.phase()
    }
    pub fn ledger(&self) -> &Ledger {
        self.table.ledger()
    }
    pub fn host_turn(&self) -> bool {
        self.current == Role::X
    }

    pub fn accept_guest(
        &mut self,
        guest_id: impl Into<String>,
        stake: i64,
    ) -> Result<RoomEvent, RoomError> {
        let guest_id = self.table.accept_guest(guest_id, stake)?;
        Ok(self
            .table
            .join_events(&guest_id, GameKind::Hangman, |h| self.state_msg(h)))
    }

    pub fn handle(&mut self, from_host: bool, msg: ClientMsg) -> Result<RoomEvent, RoomError> {
        match msg {
            hello @ ClientMsg::Hello { .. } => {
                let guest = self.table.hello(from_host, GameKind::Hangman, hello)?;
                Ok(self
                    .table
                    .join_events(&guest, GameKind::Hangman, |h| self.state_msg(h)))
            }
            ClientMsg::Guess { letter } => self.guess(from_host, letter),
            ClientMsg::Place { .. } => Err(RoomError::WrongAction),
            ClientMsg::Resign => self.resign(from_host),
            ClientMsg::Ping => Ok(self.table.ping(from_host)),
        }
    }

    /// Peer dropped mid-match: the disconnected side forfeits.
    pub fn disconnect(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        self.resign(from_host)
    }

    fn guess(&mut self, from_host: bool, letter: char) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        let actor = self.table.role_of(from_host)?;
        if self.current != actor {
            return Err(RoomError::NotYourTurn);
        }
        match self.game.guess(letter) {
            Ok(hit) => {
                let ch = letter.to_uppercase().next().unwrap_or(letter);
                self.last_guess = Some(ch);
                self.last_hit = Some(hit);
                if self.game.is_won() {
                    return self.finish(Some(actor));
                }
                if self.game.is_lost() {
                    return self.finish(Some(actor.other()));
                }
                if !hit {
                    self.current = actor.other();
                }
                Ok(self.table.broadcast_state(|h| self.state_msg(h)))
            }
            Err(GuessError::AlreadyGuessed) | Err(GuessError::NotLetter) => {
                Err(RoomError::IllegalMove)
            }
            Err(GuessError::GameOver) => Err(RoomError::WrongPhase),
        }
    }

    fn resign(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        let winner = if from_host { Role::O } else { Role::X };
        self.finish(Some(winner))
    }

    fn finish(&mut self, winner: Option<Role>) -> Result<RoomEvent, RoomError> {
        let word = Some(self.game.word().to_string());
        let host = self.state_msg(true);
        let guest = self.state_msg(false);
        self.table.finish(winner, word, host, guest)
    }

    fn state_msg(&self, for_host: bool) -> ServerMsg {
        let you = if for_host { Role::X } else { Role::O };
        let status = if self.game.is_won() {
            "won"
        } else if self.game.is_lost() {
            "lost"
        } else {
            "ongoing"
        };
        ServerMsg::HangmanState {
            display: self.game.display_word(),
            guessed: self.game.display_guessed(),
            attempts_left: self.game.attempts_left() as u32,
            max_attempts: self.game.max_attempts() as u32,
            current: self.current,
            status: status.into(),
            pot: self.table.ledger().open_pot(self.table.match_id()),
            your_turn: self.table.phase() == Phase::Playing && self.current == you,
            last_guess: self.last_guess,
            last_hit: self.last_hit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::PROTOCOL_VERSION;
    use bet_core::ledger::Ledger;

    fn room_with(word: &str, attempts: usize) -> HangmanRoom {
        let mut led = Ledger::new(100);
        led.ensure_player("host");
        led.ensure_player("guest");
        HangmanRoom::new("ABC123", "host", 10, led, word, attempts).unwrap()
    }

    fn hello() -> ClientMsg {
        ClientMsg::Hello {
            player_id: "guest".into(),
            room_code: "ABC123".into(),
            stake: 10,
            proto: PROTOCOL_VERSION,
            game: GameKind::Hangman,
        }
    }

    #[test]
    fn host_solves_word_and_settles() {
        let mut r = room_with("BET", 6);
        r.handle(false, hello()).unwrap();
        assert_eq!(r.ledger().open_pot(r.match_id()), 20);

        r.handle(true, ClientMsg::Guess { letter: 'B' }).unwrap();
        assert_eq!(r.current, Role::X);
        r.handle(true, ClientMsg::Guess { letter: 'E' }).unwrap();
        let ev = r.handle(true, ClientMsg::Guess { letter: 'T' }).unwrap();
        assert_eq!(r.phase(), Phase::Ended);
        assert!(ev.to_host.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::X),
                word: Some(w),
                ..
            } if w == "BET"
        )));
        assert_eq!(r.ledger().balance("host"), 110);
        assert_eq!(r.ledger().balance("guest"), 90);
    }

    #[test]
    fn miss_hands_turn_to_guest() {
        let mut r = room_with("BET", 6);
        r.handle(false, hello()).unwrap();
        r.handle(true, ClientMsg::Guess { letter: 'X' }).unwrap();
        assert_eq!(r.current, Role::O);
        assert_eq!(r.game.attempts_left(), 5);
        assert!(matches!(
            r.handle(true, ClientMsg::Guess { letter: 'Y' }),
            Err(RoomError::NotYourTurn)
        ));
        r.handle(false, ClientMsg::Guess { letter: 'B' }).unwrap();
        assert_eq!(r.current, Role::O);
        assert_eq!(r.game.display_word(), "B__");
    }

    #[test]
    fn last_miss_awards_opponent() {
        let mut r = room_with("BET", 1);
        r.handle(false, hello()).unwrap();
        let ev = r.handle(true, ClientMsg::Guess { letter: 'Z' }).unwrap();
        assert_eq!(r.phase(), Phase::Ended);
        assert!(ev.to_guest.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::O),
                word: Some(w),
                ..
            } if w == "BET"
        )));
        assert_eq!(r.ledger().balance("guest"), 110);
        assert_eq!(r.ledger().balance("host"), 90);
    }

    #[test]
    fn secret_word_stays_off_the_wire_until_end() {
        let mut r = room_with("SECRET", 6);
        let ev = r.handle(false, hello()).unwrap();
        for m in ev.to_host.iter().chain(ev.to_guest.iter()) {
            let dump = serde_json::to_string(m).unwrap();
            assert!(!dump.contains("SECRET"), "secret leaked on join: {dump}");
        }
        let ev = r.handle(true, ClientMsg::Guess { letter: 'X' }).unwrap();
        for m in ev.to_host.iter().chain(ev.to_guest.iter()) {
            let dump = serde_json::to_string(m).unwrap();
            assert!(!dump.contains("SECRET"), "secret leaked mid-match: {dump}");
            if let ServerMsg::HangmanState { display, .. } = m {
                assert_eq!(display, "______");
            }
        }
    }

    #[test]
    fn reject_already_guessed_and_non_letter() {
        let mut r = room_with("BET", 6);
        r.handle(false, hello()).unwrap();
        r.handle(true, ClientMsg::Guess { letter: 'B' }).unwrap();
        assert!(matches!(
            r.handle(true, ClientMsg::Guess { letter: 'B' }),
            Err(RoomError::IllegalMove)
        ));
        assert!(matches!(
            r.handle(true, ClientMsg::Guess { letter: '1' }),
            Err(RoomError::IllegalMove)
        ));
        assert_eq!(r.current, Role::X);
    }

    #[test]
    fn guest_disconnect_forfeits_to_host() {
        let mut r = room_with("BET", 6);
        r.handle(false, hello()).unwrap();
        let ev = r.disconnect(false).unwrap();
        assert_eq!(r.phase(), Phase::Ended);
        assert!(ev.to_host.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::X),
                ..
            }
        )));
        assert_eq!(r.ledger().balance("host"), 110);
    }

    #[test]
    fn game_mismatch_rejected() {
        let mut r = room_with("BET", 6);
        let bad = ClientMsg::Hello {
            player_id: "guest".into(),
            room_code: "ABC123".into(),
            stake: 10,
            proto: PROTOCOL_VERSION,
            game: GameKind::Ttt,
        };
        assert!(matches!(
            r.handle(false, bad),
            Err(RoomError::GameMismatch {
                room: GameKind::Hangman,
                guest: GameKind::Ttt
            })
        ));
    }

    #[test]
    fn from_seed_is_deterministic() {
        let led = Ledger::new(100);
        let a = HangmanRoom::from_seed(
            "SEED01",
            "h",
            10,
            led.clone(),
            99,
            &["ALPHA", "BRAVO", "CHARLIE"],
            6,
        )
        .unwrap();
        let b = HangmanRoom::from_seed(
            "SEED02",
            "h",
            10,
            led,
            99,
            &["ALPHA", "BRAVO", "CHARLIE"],
            6,
        )
        .unwrap();
        assert_eq!(a.game.word(), b.game.word());
        assert_eq!(a.game.word(), "CHARLIE");
    }

    #[test]
    fn ttt_place_rejected_in_hangman_room() {
        let mut r = room_with("BET", 6);
        r.handle(false, hello()).unwrap();
        assert!(matches!(
            r.handle(true, ClientMsg::Place { index: 0 }),
            Err(RoomError::WrongAction)
        ));
    }
}
