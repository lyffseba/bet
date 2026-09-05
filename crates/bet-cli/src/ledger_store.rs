//! Persist virtual ledger under config dir (override with BET_CONFIG_DIR).

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
    if let Ok(p) = std::env::var("BET_CONFIG_DIR") {
        let p = p.trim();
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
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
    if let Ok(raw) = fs::read_to_string(&path)
        && let Ok(file) = serde_json::from_str::<LedgerFile>(&raw)
    {
        led = Ledger::new(file.default_grant);
        led.load_balances(file.balances);
    }
    led
}

pub fn save_ledger(led: &Ledger) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let path = ledger_path();
    let file = LedgerFile {
        default_grant: led.default_grant(),
        balances: led.balances().clone(),
    };
    let raw = serde_json::to_string_pretty(&file).map_err(std::io::Error::other)?;
    // Atomic replace: write temp in same dir then rename.
    let tmp = dir.join(format!(
        ".ledger.{}.tmp",
        std::process::id()
    ));
    fs::write(&tmp, raw)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn default_player_id() -> String {
    std::env::var("BET_PLAYER")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "player".into())
}

/// Merge many balances into the on-disk ledger without dropping other rows.
pub fn merge_balances(updates: &[(String, i64)]) -> std::io::Result<()> {
    if updates.is_empty() {
        return Ok(());
    }
    // Two-pass read/merge reduces lost updates when host+guest write near-simultaneously.
    let mut map = load_ledger().balances().clone();
    for (id, bal) in updates {
        map.insert(id.clone(), *bal);
    }
    let mut again = load_ledger();
    for (k, v) in again.balances() {
        map.entry(k.clone()).or_insert(*v);
    }
    for (id, bal) in updates {
        map.insert(id.clone(), *bal);
    }
    again.load_balances(map);
    save_ledger(&again)
}

