mod game;
mod lang;
mod ui;
mod wordlist;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crossterm::terminal;

static CTRL_C: AtomicBool = AtomicBool::new(false);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up Ctrl-C handler
    let ctrlc_pressed = Arc::new(AtomicBool::new(false));
    let r = ctrlc_pressed.clone();
    ctrlc::set_handler(move || {
        CTRL_C.store(true, Ordering::SeqCst);
        r.store(true, Ordering::SeqCst);
        // Try to disable raw mode if it's enabled
        let _ = terminal::disable_raw_mode();
        // Exit the process
        std::process::exit(1);
    })?;

    ui::run_menu()?;
    Ok(())
}