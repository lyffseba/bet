use std::io::{self, Write};
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    event::{self, Event},
    execute,
    style::{Print, SetForegroundColor, Color, ResetColor},
    terminal::{self, Clear, ClearType},
};

use crate::game::Hangman;

pub fn clear_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
}

pub fn wait_for_key() -> io::Result<()> {
    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }
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

pub fn get_guess() -> io::Result<Option<char>> {
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
        return Ok(None);
    }
    if let Some(c) = input.chars().next() {
        if c.is_ascii_alphabetic() {
            return Ok(Some(c));
        }
    }
    Ok(Some('\0')) // invalid
}

pub fn play_game(mut game: Hangman) -> io::Result<()> {
    terminal::enable_raw_mode()?;
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
        match get_guess()? {
            None => break,
            Some('\0') => {
                // invalid input, ignore
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
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn get_word_from_player() -> io::Result<String> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("Enter the word for the other player to guess (will not be shown):\n"),
        ResetColor,
        Print("(Type the word and press Enter)\n"),
    )?;
    // Hide input
    execute!(stdout, Hide)?;
    let mut word = String::new();
    io::stdin().read_line(&mut word)?;
    execute!(stdout, Show)?;
    terminal::disable_raw_mode()?;
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
