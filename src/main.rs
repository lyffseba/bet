mod game;
mod lang;
mod ui;
mod wordlist;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ui::run_menu()?;
    Ok(())
}
