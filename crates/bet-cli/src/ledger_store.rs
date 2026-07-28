//! Persist virtual ledger under ~/.config/bet/ledger.json

use std::fs;
use std::path::PathBuf;

use bet_core::ledger::Ledger;
use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct LedgerFile {
    default_grant: i64,
    balances: std::collections::BTreeMap<String, i64>,
}

pub fn config_dir() -> PathBuf {
    if let Some(p) = ProjectDirs::from("xyz", "lyffseba", "bet") {
        return p.config_dir().to_path_buf();
    }
    if let Some(b) = BaseDirs::new() {
        return b.home_dir().join(".config").join("bet");
    }
    PathBuf::from(".bet")
}

pub fn ledger_path() -> PathBuf {
    config_dir().join("ledger.json")
}

pub fn load_ledger() -> Ledger {
    let path = ledger_path();
    let mut led = Ledger::new(1000);
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(file) = serde_json::from_str::<LedgerFile>(&raw) {
            led = Ledger::new(file.default_grant);
            led.load_balances(file.balances);
        }
    }
    led
}

pub fn save_ledger(led: &Ledger) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let file = LedgerFile {
        default_grant: led.default_grant(),
        balances: led.balances().clone(),
    };
    let raw = serde_json::to_string_pretty(&file).map_err(std::io::Error::other)?;
    fs::write(ledger_path(), raw)
}

pub fn default_player_id() -> String {
    std::env::var("BET_PLAYER")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "player".into())
}

/// Set one player's balance without wiping other rows (best-effort vs races).
pub fn merge_player_balance(player: &str, balance: i64) -> std::io::Result<()> {
    let disk = load_ledger();
    let mut map = disk.balances().clone();
    map.insert(player.to_string(), balance);
    // Re-read once more so a concurrent host write is less likely to be lost.
    let mut again = load_ledger();
    for (k, v) in again.balances() {
        map.entry(k.clone()).or_insert(*v);
    }
    map.insert(player.to_string(), balance);
    again.load_balances(map);
    save_ledger(&again)
}
