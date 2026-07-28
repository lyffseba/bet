//! WASM surface over `bet-core` — hangman, tic-tac-toe (vs AI), ledger.
//!
//! Built for Node / agent hosts (pi, OpenCode). UI stays in TypeScript.

#![allow(clippy::unused_unit)]

use bet_core::hash::fingerprint_hex;
use bet_core::hangman::{GuessError, Hangman};
use bet_core::ledger::{Ledger, LedgerError};
use bet_core::rng::XorShift64;
use bet_core::tictactoe::{Cell, GameStatus, Player, TicTacToe};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn wasm_start() {
    // Reserved for future panic hooks.
}

#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── Hangman ───────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct WasmHangman {
    inner: Hangman,
}

#[wasm_bindgen]
impl WasmHangman {
    /// `words_json`: JSON array of strings, e.g. `["ALPHA","BRAVO"]`.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64, words_json: &str, max_attempts: u32) -> Result<WasmHangman, JsValue> {
        let words: Vec<String> = serde_json::from_str(words_json)
            .map_err(|e| JsValue::from_str(&format!("words_json: {e}")))?;
        if words.is_empty() {
            return Err(JsValue::from_str("words_json must be a non-empty array"));
        }
        let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
        let max = max_attempts.max(1) as usize;
        Ok(Self {
            inner: Hangman::from_seed(seed, &refs, max),
        })
    }

    /// Guess a letter. Returns true if the letter is in the word.
    pub fn guess(&mut self, ch: &str) -> Result<bool, JsValue> {
        let c = ch
            .chars()
            .next()
            .ok_or_else(|| JsValue::from_str("empty guess"))?;
        self.inner
            .guess(c)
            .map_err(|e| JsValue::from_str(&guess_err(e)))
    }

    pub fn display_word(&self) -> String {
        self.inner.display_word()
    }

    pub fn attempts_left(&self) -> u32 {
        self.inner.attempts_left() as u32
    }

    pub fn max_attempts(&self) -> u32 {
        self.inner.max_attempts() as u32
    }

    pub fn is_won(&self) -> bool {
        self.inner.is_won()
    }

    pub fn is_lost(&self) -> bool {
        self.inner.is_lost()
    }

    pub fn is_over(&self) -> bool {
        self.inner.is_over()
    }

    /// Secret word (use only after game over for UI).
    pub fn word(&self) -> String {
        self.inner.word().to_string()
    }

    pub fn guessed(&self) -> String {
        self.inner.display_guessed()
    }

    pub fn state_hash(&self) -> String {
        fingerprint_hex(self.inner.state_hash())
    }
}

fn guess_err(e: GuessError) -> String {
    match e {
        GuessError::NotLetter => "not_letter".into(),
        GuessError::AlreadyGuessed => "already_guessed".into(),
        GuessError::GameOver => "game_over".into(),
    }
}

// ─── Tic-tac-toe (SP vs seeded AI) ─────────────────────────────────────────

#[wasm_bindgen]
pub struct WasmTtt {
    inner: TicTacToe,
    rng: XorShift64,
}

#[wasm_bindgen]
impl WasmTtt {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> WasmTtt {
        Self {
            inner: TicTacToe::new(),
            rng: XorShift64::new(seed),
        }
    }

    /// Human (X) places at index 0..8; AI (O) replies if ongoing.
    /// Returns false if the move was illegal.
    pub fn make_move_vs_ai(&mut self, index: u32) -> bool {
        self.inner.make_move_vs_ai(index as usize, &mut self.rng)
    }

    /// Pure multiplayer place: "x" | "o" at index.
    pub fn place(&mut self, player: &str, index: u32) -> bool {
        let p = match player.to_ascii_lowercase().as_str() {
            "x" => Player::X,
            "o" => Player::O,
            _ => return false,
        };
        self.inner.place(p, index as usize)
    }

    pub fn reset(&mut self) {
        self.inner.reset_board();
    }

    /// 9-char board: `.` empty, `X`, `O`.
    pub fn board(&self) -> String {
        self.inner
            .board
            .iter()
            .map(|c| match c {
                Cell::Empty => '.',
                Cell::Occupied(Player::X) => 'X',
                Cell::Occupied(Player::O) => 'O',
            })
            .collect()
    }

    /// `ongoing` | `win_x` | `win_o` | `draw`
    pub fn status(&self) -> String {
        match self.inner.status {
            GameStatus::Ongoing => "ongoing".into(),
            GameStatus::Win(Player::X) => "win_x".into(),
            GameStatus::Win(Player::O) => "win_o".into(),
            GameStatus::Draw => "draw".into(),
        }
    }

    /// `x` | `o`
    pub fn current(&self) -> String {
        match self.inner.current {
            Player::X => "x".into(),
            Player::O => "o".into(),
        }
    }

    pub fn state_hash(&self) -> String {
        fingerprint_hex(self.inner.state_hash())
    }
}

// ─── Ledger ────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct WasmLedger {
    inner: Ledger,
}

#[wasm_bindgen]
impl WasmLedger {
    #[wasm_bindgen(constructor)]
    pub fn new(default_grant: i32) -> WasmLedger {
        Self {
            inner: Ledger::new(default_grant as i64),
        }
    }

    pub fn ensure_player(&mut self, id: &str) {
        self.inner.ensure_player(id.to_string());
    }

    pub fn balance(&self, id: &str) -> i32 {
        self.inner.balance(id) as i32
    }

    pub fn stake(&mut self, match_id: &str, player: &str, amount: i32) -> Result<(), JsValue> {
        self.inner
            .stake(match_id, player.to_string(), amount as i64)
            .map_err(|e| JsValue::from_str(&ledger_err(e)))
    }

    pub fn settle(&mut self, match_id: &str, winner: Option<String>) -> Result<i32, JsValue> {
        self.inner
            .settle(match_id, winner.as_deref())
            .map(|p| p as i32)
            .map_err(|e| JsValue::from_str(&ledger_err(e)))
    }

    pub fn open_pot(&self, match_id: &str) -> i32 {
        self.inner.open_pot(match_id) as i32
    }

    /// JSON object map of balances.
    pub fn balances_json(&self) -> String {
        serde_json::to_string(self.inner.balances()).unwrap_or_else(|_| "{}".into())
    }
}

fn ledger_err(e: LedgerError) -> String {
    format!("{e:?}")
}

// ─── Goldens (shared with Rust tests) ──────────────────────────────────────

/// Deterministic hangman hash path for cross-language goldens.
#[wasm_bindgen]
pub fn golden_hangman_hash(seed: u64, words_json: &str) -> Result<String, JsValue> {
    let mut h = WasmHangman::new(seed, words_json, 6)?;
    let _ = h.guess("A");
    let _ = h.guess("X");
    Ok(h.state_hash())
}

#[wasm_bindgen]
pub fn golden_ttt_hash(seed: u64) -> String {
    let mut g = WasmTtt::new(seed);
    g.make_move_vs_ai(4);
    g.state_hash()
}
