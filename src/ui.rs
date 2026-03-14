use std::io::{self, Write};
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Print, SetForegroundColor, Color, ResetColor},
    terminal::{self, Clear, ClearType},
};

use crate::game::Hangman;

pub fn clear_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
}

/// Wait for any key press and return the key event.
fn wait_for_key_event() -> io::Result<KeyEvent> {
    loop {
        if let Event::Key(key) = event::read()? {
            return Ok(key);
        }
    }
}

/// Wait for any key press (ignores the key). Used for "Press any key to continue".
pub fn wait_for_key() -> io::Result<()> {
    wait_for_key_event()?;
    Ok(())
}

pub fn draw_hangman(attempts_left: usize, max_attempts: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    let stage = match max_attempts - attempts_left {
        0 => "
  +---+
  |   |
      |
      |
      |
      |
=========",
        1 => "
  +---+
  |   |
  O   |
      |
      |
      |
=========",
        2 => "
  +---+
  |   |
  O   |
  |   |
      |
      |
=========",
        3 => "
  +---+
  |   |
  O   |
 /|   |
      |
      |
=========",
        4 => "
  +---+
  |   |
  O   |
 /|\\  |
      |
      |
=========",
        5 => "
  +---+
  |   |
  O   |
 /|\\  |
 /    |
      |
=========",
        _ => "
  +---+
  |   |
  O   |
 /|\\  |
 / \\  |
      |
=========",
    };
    execute!(
        stdout,
        SetForegroundColor(Color::Yellow),
        Print(stage),
        ResetColor,
        Print("\n\n")
    )?;
    Ok(())
}

pub fn draw_game_state(game: &Hangman) -> io::Result<()> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("HANGMAN\n\n"),
        ResetColor
    )?;
    draw_hangman(game.attempts_left(), game.max_attempts())?;
    execute!(
        stdout,
        Print("Word: "),
        SetForegroundColor(Color::Green),
        Print(game.display_word()),
        ResetColor,
        Print("\n\nGuessed letters: "),
        SetForegroundColor(Color::Magenta),
        Print(game.display_guessed()),
        ResetColor,
        Print("\n\nAttempts left: "),
        SetForegroundColor(Color::Red),
        Print(game.attempts_left()),
        ResetColor,
        Print("\n\n")
    )?;
    Ok(())
}

/// Read a single key press, returning the character if it's a letter.
/// Returns None if the user wants to quit (e.g., Esc).
fn read_single_char() -> io::Result<Option<char>> {
    let key = wait_for_key_event()?;
    match key.code {
        KeyCode::Char(c) if c.is_ascii_alphabetic() => Ok(Some(c)),
        KeyCode::Esc => Ok(None), // allow Escape to quit
        _ => Ok(Some('\0')), // ignore other keys (like arrows)
    }
}

pub fn play_game(mut game: Hangman) -> io::Result<()> {
    // Use a RAII guard for raw mode to ensure it's always restored.
    let raw_guard = terminal::enable_raw_mode();
    let raw = raw_guard.is_ok();
    if !raw {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print("Note: Raw mode not available, using line input (Enter after each letter).\n"),
            ResetColor
        )?;
        // If raw mode failed, we cannot use event-driven input.
        // Fall back to line-based input (similar to before but with proper cleanup).
        return play_game_fallback(game);
    }

    loop {
        draw_game_state(&game)?;
        if game.is_won() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("Congratulations! You won!\n"),
                ResetColor,
                Print("Press any key to continue..."),
            )?;
            wait_for_key()?;
            break;
        }
        if game.is_lost() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("Game over! The word was: "),
                SetForegroundColor(Color::Yellow),
                Print(game.word()),
                ResetColor,
                Print("\nPress any key to continue..."),
            )?;
            wait_for_key()?;
            break;
        }
        match read_single_char()? {
            None => break, // Escape pressed
            Some('\0') => {
                // ignore non-letter keys
            }
            Some(letter) => {
                match game.guess(letter) {
                    Ok(true) => {
                        // correct guess, continue
                    }
                    Ok(false) => {
                        // wrong guess, attempts decreased
                    }
                    Err(msg) => {
                        let mut stdout = io::stdout();
                        execute!(
                            stdout,
                            SetForegroundColor(Color::Yellow),
                            Print(format!("{}\n", msg)),
                            ResetColor,
                        )?;
                        wait_for_key()?;
                    }
                }
            }
        }
    }
    // raw_guard will be dropped, disabling raw mode automatically.
    Ok(())
}

/// Fallback for when raw mode is not available (e.g., piped input).
/// Uses line-based input.
fn play_game_fallback(mut game: Hangman) -> io::Result<()> {
    loop {
        draw_game_state(&game)?;
        if game.is_won() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("Congratulations! You won!\n"),
                ResetColor,
                Print("Press Enter to continue..."),
            )?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            break;
        }
        if game.is_lost() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("Game over! The word was: "),
                SetForegroundColor(Color::Yellow),
                Print(game.word()),
                ResetColor,
                Print("\nPress Enter to continue..."),
            )?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            break;
        }
        // Fallback line input
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::White),
            Print("Enter a letter (or 'quit' to exit): "),
            ResetColor
        )?;
        stdout.flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();
        if input == "quit" || input == "q" {
            break;
        }
        if let Some(c) = input.chars().next() {
            if c.is_ascii_alphabetic() {
                match game.guess(c) {
                    Ok(true) => {}
                    Ok(false) => {}
                    Err(msg) => {
                        execute!(
                            stdout,
                            SetForegroundColor(Color::Yellow),
                            Print(format!("{}\n", msg)),
                            ResetColor,
                        )?;
                        let mut dummy = String::new();
                        io::stdin().read_line(&mut dummy)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn get_word_from_player() -> io::Result<String> {
    let raw_guard = terminal::enable_raw_mode();
    let raw = raw_guard.is_ok();
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("Enter the word for the other player to guess (will not be shown):\n"),
        ResetColor,
        Print("(Type the word and press Enter)\n"),
    )?;
    // Hide input only if raw mode enabled
    if raw {
        execute!(stdout, Hide)?;
    }
    let mut word = String::new();
    io::stdin().read_line(&mut word)?;
    if raw {
        execute!(stdout, Show)?;
    }
    // raw_guard dropped, disabling raw mode if enabled.
    Ok(word.trim().to_string())
}

pub fn draw_menu() -> io::Result<()> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("HANGMAN GAME\n\n"),
        ResetColor,
        Print("1. Solo (random movie)\n"),
        Print("2. Multiplayer (one player sets word)\n"),
        Print("3. Quit\n\n"),
        SetForegroundColor(Color::White),
        Print("Choose option: "),
        ResetColor,
    )?;
    stdout.flush()?;
    Ok(())
}

pub fn run_menu() -> io::Result<()> {
    loop {
        draw_menu()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        match choice {
            "1" => {
                let game = Hangman::random();
                play_game(game)?;
            }
            "2" => {
                let word = get_word_from_player()?;
                if word.is_empty() {
                    let game = Hangman::random();
                    play_game(game)?;
                } else {
                    let game = Hangman::new(&word, 6);
                    play_game(game)?;
                }
            }
            "3" | "quit" | "q" => break,
            _ => {
                // ignore
            }
        }
    }
    Ok(())
}
