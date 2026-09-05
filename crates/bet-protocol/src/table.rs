//! Shared 1v1 virtual-stake table: join, ledger, settle.
//!
//! Game rooms (tic-tac-toe, hangman) own rules + turn; this type owns
//! identity, pot, and phase so those rooms cannot drift apart.

use bet_core::ledger::Ledger;

use crate::msg::{ClientMsg, GameKind, Role, ServerMsg, PROTOCOL_VERSION};
use crate::room::{normalize_code, Phase, RoomError, RoomEvent};

/// Host + guest + virtual pot. No game rules live here.
pub struct Table {
    room_code: String,
    match_id: String,
    host_id: String,
    guest_id: Option<String>,
    stake: i64,
    phase: Phase,
    ledger: Ledger,
}

impl Table {
    pub fn room_code(&self) -> &str {
        &self.room_code
    }
    pub fn match_id(&self) -> &str {
        &self.match_id
    }
    pub fn host_id(&self) -> &str {
        &self.host_id
    }
    pub fn guest_id(&self) -> Option<&str> {
        self.guest_id.as_deref()
    }
    pub fn stake(&self) -> i64 {
        self.stake
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn open(
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
            ledger,
        })
    }

    pub fn accept_guest(
        &mut self,
        guest_id: impl Into<String>,
        stake: i64,
    ) -> Result<String, RoomError> {
        if self.phase != Phase::WaitingForGuest {
            return Err(RoomError::AlreadyStarted);
        }
        if self.guest_id.is_some() {
            return Err(RoomError::Full);
        }
        if stake != self.stake {
            return Err(RoomError::StakeMismatch {
                need: self.stake,
                got: stake,
            });
        }
        let guest_id = guest_id.into();
        if guest_id == self.host_id {
            return Err(RoomError::SamePlayer);
        }
        self.ledger.ensure_player(guest_id.clone());
        self.ledger.stake(&self.match_id, guest_id.clone(), stake)?;
        self.guest_id = Some(guest_id.clone());
        self.phase = Phase::Playing;
        Ok(guest_id)
    }

    /// Validate Hello and seat the guest. Returns the accepted guest id.
    pub fn hello(
        &mut self,
        from_host: bool,
        expected: GameKind,
        msg: ClientMsg,
    ) -> Result<String, RoomError> {
        let ClientMsg::Hello {
            player_id,
            room_code,
            stake,
            proto,
            game,
        } = msg
        else {
            return Err(RoomError::WrongPhase);
        };
        if from_host {
            return Err(RoomError::WrongPhase);
        }
        if proto != PROTOCOL_VERSION {
            return Err(RoomError::BadProto(proto));
        }
        if game != expected {
            return Err(RoomError::GameMismatch {
                room: expected,
                guest: game,
            });
        }
        let code = normalize_code(room_code)?;
        if code != self.room_code {
            return Err(RoomError::BadCode);
        }
        self.accept_guest(player_id, stake)
    }

    pub fn welcome(&self, role: Role, game: GameKind) -> ServerMsg {
        let player_id = match role {
            Role::X => self.host_id.clone(),
            Role::O => self.guest_id.clone().unwrap_or_default(),
        };
        ServerMsg::Welcome {
            player_id,
            role,
            stake: self.stake,
            match_id: self.match_id.clone(),
            room_code: self.room_code.clone(),
            proto: PROTOCOL_VERSION,
            game,
        }
    }

    pub fn join_events(
        &self,
        guest_id: &str,
        game: GameKind,
        mut state: impl FnMut(bool) -> ServerMsg,
    ) -> RoomEvent {
        let mut ev = RoomEvent::empty();
        ev.to_host.push(ServerMsg::PeerJoined {
            player_id: guest_id.to_string(),
        });
        ev.to_host.push(self.welcome(Role::X, game));
        ev.to_guest.push(self.welcome(Role::O, game));
        ev.to_host.push(state(true));
        ev.to_guest.push(state(false));
        ev
    }

    pub fn ping(&self, from_host: bool) -> RoomEvent {
        let mut ev = RoomEvent::empty();
        if from_host {
            ev.to_host.push(ServerMsg::Pong);
        } else {
            ev.to_guest.push(ServerMsg::Pong);
        }
        ev
    }

    pub fn role_of(&self, from_host: bool) -> Result<Role, RoomError> {
        if from_host {
            Ok(Role::X)
        } else if self.guest_id.is_some() {
            Ok(Role::O)
        } else {
            Err(RoomError::UnknownPlayer)
        }
    }

    pub fn must_be_playing(&self) -> Result<(), RoomError> {
        if self.phase == Phase::Playing {
            Ok(())
        } else {
            Err(RoomError::WrongPhase)
        }
    }

    pub fn broadcast_state(&self, mut state: impl FnMut(bool) -> ServerMsg) -> RoomEvent {
        let mut ev = RoomEvent::empty();
        ev.to_host.push(state(true));
        ev.to_guest.push(state(false));
        ev
    }

    /// Close the pot. `word` is hangman-only (omit for ttt).
    /// Callers must pass already-built state snapshots so we do not re-borrow `self`.
    pub fn finish(
        &mut self,
        winner: Option<Role>,
        word: Option<String>,
        state_host: ServerMsg,
        state_guest: ServerMsg,
    ) -> Result<RoomEvent, RoomError> {
        self.phase = Phase::Ended;
        let winner_id = match winner {
            Some(Role::X) => Some(self.host_id.clone()),
            Some(Role::O) => self.guest_id.clone(),
            None => None,
        };
        let pot_before = self.ledger.open_pot(&self.match_id);
        self.ledger.settle(&self.match_id, winner_id.as_deref())?;
        let bal_h = self.ledger.balance(&self.host_id);
        let bal_g = self
            .guest_id
            .as_ref()
            .map(|g| self.ledger.balance(g))
            .unwrap_or(0);
        let ended = ServerMsg::MatchEnded {
            winner,
            pot: pot_before,
            winner_id,
            host_id: self.host_id.clone(),
            guest_id: self.guest_id.clone(),
            balance_host: bal_h,
            balance_guest: bal_g,
            word,
        };
        let mut ev = RoomEvent::empty();
        ev.to_host.push(state_host);
        ev.to_guest.push(state_guest);
        ev.to_host.push(ended.clone());
        ev.to_guest.push(ended);
        Ok(ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bet_core::ledger::Ledger;

    #[test]
    fn stake_mismatch_is_typed() {
        let mut t = Table::open("ABCD", "h", 10, Ledger::new(100)).unwrap();
        assert!(matches!(
            t.accept_guest("g", 5),
            Err(RoomError::StakeMismatch { need: 10, got: 5 })
        ));
    }

    #[test]
    fn hello_game_mismatch_is_typed() {
        let mut t = Table::open("ABCD", "h", 10, Ledger::new(100)).unwrap();
        let hello = ClientMsg::Hello {
            player_id: "g".into(),
            room_code: "ABCD".into(),
            stake: 10,
            proto: PROTOCOL_VERSION,
            game: GameKind::Hangman,
        };
        assert!(matches!(
            t.hello(false, GameKind::Ttt, hello),
            Err(RoomError::GameMismatch {
                room: GameKind::Ttt,
                guest: GameKind::Hangman
            })
        ));
    }
}
