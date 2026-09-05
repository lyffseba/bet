//! Wire messages (JSON). One object per line on TCP.
//!
//! Additive rules (keep `PROTOCOL_VERSION` unless a field is removed/renamed):
//! - `Hello.game` / `Welcome.game` default to `ttt`
//! - hangman-only variants are never sent to a ttt client
//! - the secret hangman word is never put on the wire until `MatchEnded`

use bet_core::tictactoe::{Cell, GameStatus, Player};
use serde::{Deserialize, Serialize};

/// Wire protocol version. Bump when breaking Hello/State/MatchEnded shape.
pub const PROTOCOL_VERSION: u32 = 1;

/// Which game a room is playing. Missing `Hello.game` deserializes as `Ttt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameKind {
    #[default]
    Ttt,
    Hangman,
}

impl GameKind {
    pub fn as_str(self) -> &'static str {
        match self {
            GameKind::Ttt => "ttt",
            GameKind::Hangman => "hangman",
        }
    }
}

impl std::fmt::Display for GameKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for GameKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ttt" | "tictactoe" | "tic-tac-toe" | "tic" => Ok(GameKind::Ttt),
            "hangman" | "hang" | "hm" => Ok(GameKind::Hangman),
            other => Err(format!(
                "unknown game `{other}` (expected ttt or hangman)"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    X,
    O,
}

impl Role {
    pub fn other(self) -> Self {
        match self {
            Role::X => Role::O,
            Role::O => Role::X,
        }
    }
}

impl From<Player> for Role {
    fn from(p: Player) -> Self {
        match p {
            Player::X => Role::X,
            Player::O => Role::O,
        }
    }
}

impl From<Role> for Player {
    fn from(r: Role) -> Self {
        match r {
            Role::X => Player::X,
            Role::O => Player::O,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    /// Guest announces identity and stake intent.
    Hello {
        player_id: String,
        room_code: String,
        stake: i64,
        /// Optional; missing treated as 1 for older clients.
        #[serde(default = "default_proto")]
        proto: u32,
        /// Optional; missing treated as tic-tac-toe.
        #[serde(default)]
        game: GameKind,
    },
    /// Place a mark at board index 0..8 (tic-tac-toe).
    Place { index: u8 },
    /// Guess a single letter (hangman). Host rejects non-letters / already-guessed.
    Guess { letter: char },
    Resign,
    Ping,
}

fn default_proto() -> u32 {
    PROTOCOL_VERSION
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Welcome {
        player_id: String,
        role: Role,
        stake: i64,
        match_id: String,
        room_code: String,
        proto: u32,
        #[serde(default)]
        game: GameKind,
    },
    PeerJoined {
        player_id: String,
    },
    State {
        board: [char; 9],
        current: Role,
        status: String,
        pot: i64,
        your_turn: bool,
        wins_x: u32,
        wins_o: u32,
        draws: u32,
    },
    /// Public hangman view. Never includes the secret word while play is ongoing.
    HangmanState {
        /// Masked word (`_` for unrevealed letters; punctuation/spaces kept).
        display: String,
        guessed: String,
        attempts_left: u32,
        max_attempts: u32,
        current: Role,
        status: String,
        pot: i64,
        your_turn: bool,
        last_guess: Option<char>,
        last_hit: Option<bool>,
    },
    MatchEnded {
        winner: Option<Role>,
        pot: i64,
        /// Winner player_id if any
        winner_id: Option<String>,
        host_id: String,
        guest_id: Option<String>,
        balance_host: i64,
        balance_guest: i64,
        /// Revealed only after the match ends (hangman). Absent for ttt.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        word: Option<String>,
    },
    Error {
        message: String,
    },
    Pong,
}

pub fn cell_char(c: Cell) -> char {
    match c {
        Cell::Empty => '.',
        Cell::Occupied(Player::X) => 'X',
        Cell::Occupied(Player::O) => 'O',
    }
}

pub fn status_str(s: GameStatus) -> String {
    match s {
        GameStatus::Ongoing => "ongoing".into(),
        GameStatus::Win(Player::X) => "win_x".into(),
        GameStatus::Win(Player::O) => "win_o".into(),
        GameStatus::Draw => "draw".into(),
    }
}

pub fn encode_line(msg: &impl Serialize) -> Result<String, serde_json::Error> {
    let mut s = serde_json::to_string(msg)?;
    s.push('\n');
    Ok(s)
}

pub fn decode_client_line(line: &str) -> Result<ClientMsg, serde_json::Error> {
    serde_json::from_str(line.trim())
}

pub fn decode_server_line(line: &str) -> Result<ServerMsg, serde_json::Error> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_without_game_defaults_to_ttt() {
        let raw = r#"{"type":"welcome","player_id":"a","role":"x","stake":10,"match_id":"m","room_code":"AB12","proto":1}"#;
        let msg: ServerMsg = serde_json::from_str(raw).unwrap();
        match msg {
            ServerMsg::Welcome { game, .. } => assert_eq!(game, GameKind::Ttt),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn match_ended_without_word_is_none() {
        let raw = r#"{"type":"match_ended","winner":"x","pot":20,"winner_id":"a","host_id":"a","guest_id":"b","balance_host":110,"balance_guest":90}"#;
        let msg: ServerMsg = serde_json::from_str(raw).unwrap();
        match msg {
            ServerMsg::MatchEnded { word, winner, .. } => {
                assert_eq!(word, None);
                assert_eq!(winner, Some(Role::X));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn ttt_match_ended_omits_word_on_the_wire() {
        let msg = ServerMsg::MatchEnded {
            winner: Some(Role::X),
            pot: 20,
            winner_id: Some("a".into()),
            host_id: "a".into(),
            guest_id: Some("b".into()),
            balance_host: 110,
            balance_guest: 90,
            word: None,
        };
        let dump = serde_json::to_string(&msg).unwrap();
        assert!(!dump.contains("word"), "ttt must not emit word=: {dump}");
    }
}
