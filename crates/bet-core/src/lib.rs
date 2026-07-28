//! Pure, deterministic game rules and virtual-points ledger for BET.
//!
//! No filesystem, sockets, or wall-clock RNG. Inject seeds for randomness.

#![forbid(unsafe_code)]

pub mod hash;
pub mod hangman;
pub mod ledger;
pub mod rng;
pub mod tictactoe;

pub use hangman::{GuessError, Hangman};
pub use ledger::{Ledger, LedgerError, PlayerId};
pub use rng::XorShift64;
pub use tictactoe::{Cell, GameStatus, Player, TicTacToe};
