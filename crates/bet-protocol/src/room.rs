//! Host-authoritative tic-tac-toe room with virtual stakes.

use bet_core::ledger::{Ledger, LedgerError};
use bet_core::tictactoe::{GameStatus, Player, TicTacToe};

use crate::msg::{cell_char, status_str, ClientMsg, GameKind, Role, ServerMsg};
use crate::table::Table;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    WaitingForGuest,
    Playing,
    Ended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomError {
    BadCode,
    Full,
    WrongPhase,
    /// Place sent to hangman, Guess sent to ttt, etc.
    WrongAction,
    NotYourTurn,
    IllegalMove,
    Ledger(bet_core::ledger::LedgerError),
    SamePlayer,
    EmptyWord,
    EmptyWordList,
    UnknownPlayer,
    AlreadyStarted,
    BadProto(u32),
    InvalidStake,
    InvalidCode,
    GameMismatch { room: crate::msg::GameKind, guest: crate::msg::GameKind },
    StakeMismatch { need: i64, got: i64 },
}

impl std::fmt::Display for RoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomError::BadCode => write!(f, "room code mismatch"),
            RoomError::Full => write!(f, "room is full"),
            RoomError::WrongPhase => write!(f, "wrong phase for this action"),
            RoomError::WrongAction => write!(f, "action does not belong to this game"),
            RoomError::NotYourTurn => write!(f, "not your turn"),
            RoomError::IllegalMove => write!(f, "illegal move"),
            RoomError::Ledger(e) => write!(f, "ledger: {e:?}"),
            RoomError::SamePlayer => write!(f, "guest id must differ from host"),
            RoomError::EmptyWord => write!(f, "hangman word has no letters"),
            RoomError::EmptyWordList => write!(f, "empty hangman word list"),
            RoomError::UnknownPlayer => write!(f, "unknown player"),
            RoomError::AlreadyStarted => write!(f, "match already started"),
            RoomError::BadProto(v) => write!(f, "unsupported proto {v}"),
            RoomError::InvalidStake => write!(f, "stake must be > 0"),
            RoomError::InvalidCode => write!(f, "room code must be 4–8 alphanumeric"),
            RoomError::GameMismatch { room, guest } => {
                write!(f, "game mismatch: this room is {room}, guest asked for {guest}")
            }
            RoomError::StakeMismatch { need, got } => {
                write!(f, "stake mismatch: this room needs {need}, guest offered {got}")
            }
        }
    }
}

impl std::error::Error for RoomError {}

impl From<LedgerError> for RoomError {
    fn from(e: LedgerError) -> Self {
        RoomError::Ledger(e)
    }
}

#[derive(Debug, Clone)]
pub struct RoomEvent {
    /// Messages destined for host local UI (player_id == host).
    pub to_host: Vec<ServerMsg>,
    /// Messages destined for guest.
    pub to_guest: Vec<ServerMsg>,
}

impl RoomEvent {
    pub(crate) fn empty() -> Self {
        Self {
            to_host: Vec::new(),
            to_guest: Vec::new(),
        }
    }
}

/// One host + one guest, tic-tac-toe, virtual pot.
pub struct TttRoom {
    pub table: Table,
    pub game: TicTacToe,
}

impl TttRoom {
    pub fn new(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        ledger: Ledger,
    ) -> Result<Self, RoomError> {
        Ok(Self {
            table: Table::open(room_code, host_id, stake, ledger)?,
            game: TicTacToe::new(),
        })
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
        self.game.current == Player::X
    }

    pub fn accept_guest(
        &mut self,
        guest_id: impl Into<String>,
        stake: i64,
    ) -> Result<RoomEvent, RoomError> {
        let guest_id = self.table.accept_guest(guest_id, stake)?;
        Ok(self.table.join_events(&guest_id, GameKind::Ttt, |h| self.state_msg(h)))
    }

    pub fn handle(&mut self, from_host: bool, msg: ClientMsg) -> Result<RoomEvent, RoomError> {
        match msg {
            hello @ ClientMsg::Hello { .. } => {
                let guest = self.table.hello(from_host, GameKind::Ttt, hello)?;
                Ok(self.table.join_events(&guest, GameKind::Ttt, |h| self.state_msg(h)))
            }
            ClientMsg::Place { index } => self.place(from_host, index as usize),
            ClientMsg::Guess { .. } => Err(RoomError::WrongAction),
            ClientMsg::Resign => self.resign(from_host),
            ClientMsg::Ping => Ok(self.table.ping(from_host)),
        }
    }

    /// Peer dropped mid-match: the disconnected side forfeits.
    pub fn disconnect(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        self.resign(from_host)
    }

    fn place(&mut self, from_host: bool, index: usize) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        let player = Player::from(self.table.role_of(from_host)?);
        if self.game.current != player {
            return Err(RoomError::NotYourTurn);
        }
        if !self.game.place(player, index) {
            return Err(RoomError::IllegalMove);
        }
        self.after_move()
    }

    fn resign(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        self.table.must_be_playing()?;
        let winner = if from_host { Player::O } else { Player::X };
        self.game.status = GameStatus::Win(winner);
        self.finish(Some(winner))
    }

    fn after_move(&mut self) -> Result<RoomEvent, RoomError> {
        match self.game.status {
            GameStatus::Ongoing => Ok(self.table.broadcast_state(|h| self.state_msg(h))),
            GameStatus::Win(p) => self.finish(Some(p)),
            GameStatus::Draw => self.finish(None),
        }
    }

    fn finish(&mut self, winner: Option<Player>) -> Result<RoomEvent, RoomError> {
        let host = self.state_msg(true);
        let guest = self.state_msg(false);
        self.table.finish(winner.map(Role::from), None, host, guest)
    }

    fn state_msg(&self, for_host: bool) -> ServerMsg {
        let you = if for_host { Player::X } else { Player::O };
        let mut board = ['.'; 9];
        for (i, c) in self.game.board.iter().enumerate() {
            board[i] = cell_char(*c);
        }
        ServerMsg::State {
            board,
            current: Role::from(self.game.current),
            status: status_str(self.game.status),
            pot: self.table.ledger().open_pot(self.table.match_id()),
            your_turn: self.table.phase() == Phase::Playing && self.game.current == you,
            wins_x: self.game.wins,
            wins_o: self.game.losses,
            draws: self.game.draws,
        }
    }
}

/// Uppercase A–Z / 2–9, length 4–8 (no ambiguous 0/O/1/I required but allowed if caller chooses).
pub fn normalize_code(raw: impl AsRef<str>) -> Result<String, RoomError> {
    let s: String = raw
        .as_ref()
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if s.len() < 4 || s.len() > 8 {
        return Err(RoomError::InvalidCode);
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bet_core::ledger::Ledger;

    fn room() -> TttRoom {
        let mut led = Ledger::new(100);
        led.ensure_player("host");
        led.ensure_player("guest");
        TttRoom::new("ABC123", "host", 10, led).unwrap()
    }

    #[test]
    fn full_match_host_wins_and_settles() {
        let mut r = room();
        assert_eq!(r.ledger().balance("host"), 90); // staked
        r.accept_guest("guest", 10).unwrap();
        assert_eq!(r.ledger().balance("guest"), 90);
        assert_eq!(r.ledger().open_pot(r.match_id()), 20);

        // X 0, O 3, X 1, O 4, X 2 => X wins top row
        r.handle(true, ClientMsg::Place { index: 0 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 3 }).unwrap();
        r.handle(true, ClientMsg::Place { index: 1 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 4 }).unwrap();
        let ev = r.handle(true, ClientMsg::Place { index: 2 }).unwrap();
        assert_eq!(r.phase(), Phase::Ended);
        assert!(
            ev.to_host
                .iter()
                .any(|m| matches!(m, ServerMsg::MatchEnded { winner: Some(Role::X), .. }))
        );
        assert_eq!(r.ledger().balance("host"), 110);
        assert_eq!(r.ledger().balance("guest"), 90);
    }

    #[test]
    fn reject_out_of_turn() {
        let mut r = room();
        r.accept_guest("guest", 10).unwrap();
        assert!(matches!(
            r.handle(false, ClientMsg::Place { index: 0 }),
            Err(RoomError::NotYourTurn)
        ));
    }

    #[test]
    fn stake_mismatch() {
        let mut r = room();
        assert!(r.accept_guest("guest", 5).is_err());
    }

    #[test]
    fn guest_disconnect_forfeits_to_host() {
        let mut r = room();
        r.accept_guest("guest", 10).unwrap();
        let ev = r.disconnect(false).unwrap(); // guest drops
        assert_eq!(r.phase(), Phase::Ended);
        assert!(ev.to_host.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::X),
                ..
            }
        )));
        assert_eq!(r.ledger().balance("host"), 110);
        assert_eq!(r.ledger().balance("guest"), 90);
    }

    #[test]
    fn draw_refunds_stakes() {
        let mut r = room();
        r.accept_guest("guest", 10).unwrap();
        // X0 O1 X2 O3 X4 O5 X6 O7 X8 — not a forced draw layout; craft via places carefully:
        // Classic draw:
        // X O X
        // X O O
        // O X X
        let seq = [
            (true, 0u8),
            (false, 1),
            (true, 2),
            (false, 4),
            (true, 3),
            (false, 5),
            (true, 7),
            (false, 6),
            (true, 8),
        ];
        for (h, i) in seq {
            r.handle(h, ClientMsg::Place { index: i }).unwrap();
        }
        assert_eq!(r.phase(), Phase::Ended);
        assert_eq!(r.ledger().balance("host"), 100);
        assert_eq!(r.ledger().balance("guest"), 100);
    }

    #[test]
    fn normalize_code_rules() {
        assert_eq!(normalize_code("ab12").unwrap(), "AB12");
        assert!(normalize_code("ab").is_err());
        assert!(normalize_code("").is_err());
    }
}
