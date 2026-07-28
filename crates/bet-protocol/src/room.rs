//! Host-authoritative tic-tac-toe room with virtual stakes.

use bet_core::ledger::{Ledger, LedgerError};
use bet_core::tictactoe::{GameStatus, Player, TicTacToe};

use crate::msg::{cell_char, status_str, ClientMsg, Role, ServerMsg};

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
    NotYourTurn,
    IllegalMove,
    Ledger(String),
    UnknownPlayer,
    AlreadyStarted,
    BadProto(u32),
    InvalidStake,
    InvalidCode,
}

impl std::fmt::Display for RoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RoomError {}

impl From<LedgerError> for RoomError {
    fn from(e: LedgerError) -> Self {
        RoomError::Ledger(format!("{e:?}"))
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
    fn empty() -> Self {
        Self {
            to_host: Vec::new(),
            to_guest: Vec::new(),
        }
    }
}

/// One host + one guest, tic-tac-toe, virtual pot.
pub struct TttRoom {
    pub room_code: String,
    pub match_id: String,
    pub host_id: String,
    pub guest_id: Option<String>,
    pub stake: i64,
    pub phase: Phase,
    pub game: TicTacToe,
    pub ledger: Ledger,
}

impl TttRoom {
    pub fn new(
        room_code: impl Into<String>,
        host_id: impl Into<String>,
        stake: i64,
        mut ledger: Ledger,
    ) -> Result<Self, RoomError> {
        if stake <= 0 {
            return Err(RoomError::InvalidStake);
        }
        let room_code = normalize_code(room_code.into())?;
        let host_id = host_id.into();
        if host_id.trim().is_empty() {
            return Err(RoomError::UnknownPlayer);
        }
        let match_id = format!("m-{}", room_code.to_lowercase());
        ledger.ensure_player(host_id.clone());
        ledger.stake(&match_id, host_id.clone(), stake)?;
        Ok(Self {
            room_code,
            match_id,
            host_id,
            guest_id: None,
            stake,
            phase: Phase::WaitingForGuest,
            game: TicTacToe::new(),
            ledger,
        })
    }

    pub fn guest_id(&self) -> Option<&str> {
        self.guest_id.as_deref()
    }

    pub fn accept_guest(&mut self, guest_id: impl Into<String>, stake: i64) -> Result<RoomEvent, RoomError> {
        if self.phase != Phase::WaitingForGuest {
            return Err(RoomError::AlreadyStarted);
        }
        if self.guest_id.is_some() {
            return Err(RoomError::Full);
        }
        if stake != self.stake {
            return Err(RoomError::Ledger(format!(
                "stake mismatch: need {}, got {}",
                self.stake, stake
            )));
        }
        let guest_id = guest_id.into();
        if guest_id == self.host_id {
            return Err(RoomError::Ledger("guest id must differ from host".into()));
        }
        self.ledger.ensure_player(guest_id.clone());
        self.ledger.stake(&self.match_id, guest_id.clone(), stake)?;
        self.guest_id = Some(guest_id.clone());
        self.phase = Phase::Playing;

        let mut ev = RoomEvent::empty();
        ev.to_host.push(ServerMsg::PeerJoined {
            player_id: guest_id.clone(),
        });
        ev.to_host.push(ServerMsg::Welcome {
            player_id: self.host_id.clone(),
            role: Role::X,
            stake: self.stake,
            match_id: self.match_id.clone(),
            room_code: self.room_code.clone(),
            proto: crate::msg::PROTOCOL_VERSION,
        });
        ev.to_guest.push(ServerMsg::Welcome {
            player_id: guest_id,
            role: Role::O,
            stake: self.stake,
            match_id: self.match_id.clone(),
            room_code: self.room_code.clone(),
            proto: crate::msg::PROTOCOL_VERSION,
        });
        let state_h = self.state_msg(true);
        let state_g = self.state_msg(false);
        ev.to_host.push(state_h);
        ev.to_guest.push(state_g);
        Ok(ev)
    }

    pub fn handle(&mut self, from_host: bool, msg: ClientMsg) -> Result<RoomEvent, RoomError> {
        match msg {
            ClientMsg::Hello {
                player_id,
                room_code,
                stake,
                proto,
            } => {
                if from_host {
                    return Err(RoomError::WrongPhase);
                }
                if proto != crate::msg::PROTOCOL_VERSION {
                    return Err(RoomError::BadProto(proto));
                }
                let code = normalize_code(room_code)?;
                if code != self.room_code {
                    return Err(RoomError::BadCode);
                }
                self.accept_guest(player_id, stake)
            }
            ClientMsg::Place { index } => self.place(from_host, index as usize),
            ClientMsg::Resign => self.resign(from_host),
            ClientMsg::Ping => {
                let mut ev = RoomEvent::empty();
                if from_host {
                    ev.to_host.push(ServerMsg::Pong);
                } else {
                    ev.to_guest.push(ServerMsg::Pong);
                }
                Ok(ev)
            }
        }
    }

    /// Peer dropped mid-match: the disconnected side forfeits.
    pub fn disconnect(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        if self.phase != Phase::Playing {
            return Err(RoomError::WrongPhase);
        }
        self.resign(from_host)
    }

    fn player_of(&self, from_host: bool) -> Result<Player, RoomError> {
        if from_host {
            Ok(Player::X)
        } else if self.guest_id.is_some() {
            Ok(Player::O)
        } else {
            Err(RoomError::UnknownPlayer)
        }
    }

    fn place(&mut self, from_host: bool, index: usize) -> Result<RoomEvent, RoomError> {
        if self.phase != Phase::Playing {
            return Err(RoomError::WrongPhase);
        }
        let player = self.player_of(from_host)?;
        if self.game.current != player {
            return Err(RoomError::NotYourTurn);
        }
        if !self.game.place(player, index) {
            return Err(RoomError::IllegalMove);
        }
        self.after_move()
    }

    fn resign(&mut self, from_host: bool) -> Result<RoomEvent, RoomError> {
        if self.phase != Phase::Playing {
            return Err(RoomError::WrongPhase);
        }
        let winner = if from_host { Player::O } else { Player::X };
        self.game.status = GameStatus::Win(winner);
        self.finish(Some(winner))
    }

    fn after_move(&mut self) -> Result<RoomEvent, RoomError> {
        match self.game.status {
            GameStatus::Ongoing => {
                let mut ev = RoomEvent::empty();
                ev.to_host.push(self.state_msg(true));
                ev.to_guest.push(self.state_msg(false));
                Ok(ev)
            }
            GameStatus::Win(p) => self.finish(Some(p)),
            GameStatus::Draw => self.finish(None),
        }
    }

    fn finish(&mut self, winner: Option<Player>) -> Result<RoomEvent, RoomError> {
        self.phase = Phase::Ended;
        let winner_id = match winner {
            Some(Player::X) => Some(self.host_id.clone()),
            Some(Player::O) => self.guest_id.clone(),
            None => None,
        };
        let pot_before = self.ledger.open_pot(&self.match_id);
        self.ledger
            .settle(&self.match_id, winner_id.as_deref())?;
        let bal_h = self.ledger.balance(&self.host_id);
        let bal_g = self
            .guest_id
            .as_ref()
            .map(|g| self.ledger.balance(g))
            .unwrap_or(0);

        let ended = ServerMsg::MatchEnded {
            winner: winner.map(Role::from),
            pot: pot_before,
            winner_id,
            host_id: self.host_id.clone(),
            guest_id: self.guest_id.clone(),
            balance_host: bal_h,
            balance_guest: bal_g,
        };
        let mut ev = RoomEvent::empty();
        ev.to_host.push(self.state_msg(true));
        ev.to_guest.push(self.state_msg(false));
        ev.to_host.push(ended.clone());
        ev.to_guest.push(ended);
        Ok(ev)
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
            pot: self.ledger.open_pot(&self.match_id),
            your_turn: self.phase == Phase::Playing && self.game.current == you,
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
        assert_eq!(r.ledger.balance("host"), 90); // staked
        r.accept_guest("guest", 10).unwrap();
        assert_eq!(r.ledger.balance("guest"), 90);
        assert_eq!(r.ledger.open_pot(&r.match_id), 20);

        // X 0, O 3, X 1, O 4, X 2 => X wins top row
        r.handle(true, ClientMsg::Place { index: 0 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 3 }).unwrap();
        r.handle(true, ClientMsg::Place { index: 1 }).unwrap();
        r.handle(false, ClientMsg::Place { index: 4 }).unwrap();
        let ev = r.handle(true, ClientMsg::Place { index: 2 }).unwrap();
        assert_eq!(r.phase, Phase::Ended);
        assert!(
            ev.to_host
                .iter()
                .any(|m| matches!(m, ServerMsg::MatchEnded { winner: Some(Role::X), .. }))
        );
        assert_eq!(r.ledger.balance("host"), 110);
        assert_eq!(r.ledger.balance("guest"), 90);
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
        assert_eq!(r.phase, Phase::Ended);
        assert!(ev.to_host.iter().any(|m| matches!(
            m,
            ServerMsg::MatchEnded {
                winner: Some(Role::X),
                ..
            }
        )));
        assert_eq!(r.ledger.balance("host"), 110);
        assert_eq!(r.ledger.balance("guest"), 90);
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
        assert_eq!(r.phase, Phase::Ended);
        assert_eq!(r.ledger.balance("host"), 100);
        assert_eq!(r.ledger.balance("guest"), 100);
    }

    #[test]
    fn normalize_code_rules() {
        assert_eq!(normalize_code("ab12").unwrap(), "AB12");
        assert!(normalize_code("ab").is_err());
        assert!(normalize_code("").is_err());
    }
}
