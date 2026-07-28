//! Wire messages (JSON). One object per line on TCP.

use bet_core::tictactoe::{Cell, GameStatus, Player};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    X,
    O,
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
    /// Guest (or reconnect) announces identity and stake intent.
    Hello {
        player_id: String,
        room_code: String,
        stake: i64,
    },
    /// Place a mark at board index 0..8.
    Place { index: u8 },
    Resign,
    Ping,
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
    MatchEnded {
        winner: Option<Role>,
        pot: i64,
        /// Winner player_id if any
        winner_id: Option<String>,
        balance_host: i64,
        balance_guest: i64,
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
