//! Virtual-points ledger (v1: no real money).

use std::collections::BTreeMap;

/// Opaque player id (npub, local name, or uuid string).
pub type PlayerId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LedgerError {
    UnknownPlayer,
    InsufficientFunds,
    InvalidAmount,
    NoOpenStake,
    StakeExists,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ledger {
    balances: BTreeMap<PlayerId, i64>,
    /// match_id -> (player -> staked amount)
    open_stakes: BTreeMap<String, BTreeMap<PlayerId, i64>>,
    default_grant: i64,
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Ledger {
    pub fn new(default_grant: i64) -> Self {
        Self {
            balances: BTreeMap::new(),
            open_stakes: BTreeMap::new(),
            default_grant,
        }
    }

    pub fn ensure_player(&mut self, id: impl Into<PlayerId>) -> &mut i64 {
        let id = id.into();
        let grant = self.default_grant;
        self.balances.entry(id).or_insert(grant)
    }

    pub fn balance(&self, id: &str) -> i64 {
        self.balances.get(id).copied().unwrap_or(0)
    }

    pub fn stake(
        &mut self,
        match_id: impl Into<String>,
        player: impl Into<PlayerId>,
        amount: i64,
    ) -> Result<(), LedgerError> {
        if amount <= 0 {
            return Err(LedgerError::InvalidAmount);
        }
        let match_id = match_id.into();
        let player = player.into();
        self.ensure_player(player.clone());
        let bal = self.balances.get_mut(&player).unwrap();
        if *bal < amount {
            return Err(LedgerError::InsufficientFunds);
        }
        let stakes = self.open_stakes.entry(match_id).or_default();
        if stakes.contains_key(&player) {
            return Err(LedgerError::StakeExists);
        }
        *bal -= amount;
        stakes.insert(player, amount);
        Ok(())
    }

    /// Winner takes the full pot. If `winner` is None, refund all stakes (draw / cancel).
    pub fn settle(
        &mut self,
        match_id: &str,
        winner: Option<&str>,
    ) -> Result<i64, LedgerError> {
        let stakes = self
            .open_stakes
            .remove(match_id)
            .ok_or(LedgerError::NoOpenStake)?;
        let pot: i64 = stakes.values().sum();
        match winner {
            Some(w) => {
                self.ensure_player(w.to_string());
                *self.balances.get_mut(w).unwrap() += pot;
                Ok(pot)
            }
            None => {
                for (p, amt) in stakes {
                    self.ensure_player(p.clone());
                    *self.balances.get_mut(&p).unwrap() += amt;
                }
                Ok(0)
            }
        }
    }

    pub fn open_pot(&self, match_id: &str) -> i64 {
        self.open_stakes
            .get(match_id)
            .map(|m| m.values().sum())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stake_and_settle_winner() {
        let mut led = Ledger::new(100);
        led.stake("m1", "alice", 10).unwrap();
        led.stake("m1", "bob", 10).unwrap();
        assert_eq!(led.balance("alice"), 90);
        assert_eq!(led.balance("bob"), 90);
        assert_eq!(led.open_pot("m1"), 20);
        led.settle("m1", Some("alice")).unwrap();
        assert_eq!(led.balance("alice"), 110);
        assert_eq!(led.balance("bob"), 90);
    }

    #[test]
    fn refuse_overdraw() {
        let mut led = Ledger::new(5);
        led.ensure_player("x");
        assert_eq!(
            led.stake("m", "x", 10),
            Err(LedgerError::InsufficientFunds)
        );
    }

    #[test]
    fn draw_refunds() {
        let mut led = Ledger::new(50);
        led.stake("m", "a", 5).unwrap();
        led.stake("m", "b", 5).unwrap();
        led.settle("m", None).unwrap();
        assert_eq!(led.balance("a"), 50);
        assert_eq!(led.balance("b"), 50);
    }
}
