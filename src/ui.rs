use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    style::{Print, SetForegroundColor, Color, ResetColor},
    terminal::{self, Clear, ClearType},
};

use crate::game::{Hangman, GuessError};
use crate::lang::{Language, Lang};

#[derive(Debug)]
enum GuessResult {
    Letter(char),
    Quit,
    Timeout,
}

pub fn clear_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))
}

/// Wait for user to press Enter (line input).
fn wait_for_enter() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

fn select_language() -> io::Result<Language> {
    loop {
        clear_screen()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print("SELECT LANGUAGE / SELECCIONE IDIOMA / SELECIONE O IDIOMA\n\n"),
            ResetColor,
            Print("1. English\n"),
            Print("2. Español\n"),
            Print("3. Português\n\n"),
            SetForegroundColor(Color::White),
            Print("Choose option / Elige una opción / Escolha uma opção: "),
            ResetColor,
        )?;
        stdout.flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim() {
            "1" => return Ok(Language::English),
            "2" => return Ok(Language::Spanish),
            "3" => return Ok(Language::Portuguese),
            _ => {}
        }
    }
}

/// Hangman ASCII art for each stage (0..6).
/// Stage 0: empty gallows
/// Stage 1: head
/// Stage 2: torso
/// Stage 3: left arm
/// Stage 4: right arm
/// Stage 5: left leg
/// Stage 6: right leg (game over)
const HANGMAN_ART: [&str; 7] = [
    // Stage 0
    "
  ┌───┐
  │   │
      │
      │
      │
      │
═══════════",
    // Stage 1
    "
  ┌───┐
  │   │
  O   │
      │
      │
      │
═══════════",
    // Stage 2
    "
  ┌───┐
  │   │
  O   │
  │   │
      │
      │
═══════════",
    // Stage 3
    "
  ┌───┐
  │   │
  O   │
 /│   │
      │
      │
═══════════",
    // Stage 4
    "
  ┌───┐
  │   │
  O   │
 /│\\  │
      │
      │
═══════════",
    // Stage 5
    "
  ┌───┐
  │   │
  O   │
 /│\\  │
 /    │
      │
═══════════",
    // Stage 6
    "
  ┌───┐
  │   │
  O   │
 /│\\  │
 / \\  │
      │
═══════════",
];

pub fn draw_hangman(attempts_left: usize, max_attempts: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    let stage = max_attempts - attempts_left;
    let art = HANGMAN_ART[stage.min(6)];
    execute!(
        stdout,
        SetForegroundColor(Color::Yellow),
        Print(art),
        ResetColor,
        Print("\n\n")
    )?;
    Ok(())
}

/// Draw the hangman with a short delay after drawing a new body part.
/// Returns true if a new part was drawn (i.e., stage changed).
fn draw_hangman_with_delay(
    attempts_left: usize,
    max_attempts: usize,
    previous_attempts_left: usize,
) -> io::Result<bool> {
    let stage = max_attempts - attempts_left;
    let prev_stage = max_attempts - previous_attempts_left;
    if stage > prev_stage {
        // New body part added: draw with a short delay
        clear_screen()?;
        draw_hangman(attempts_left, max_attempts)?;
        thread::sleep(Duration::from_millis(200));
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn draw_game_state(game: &Hangman, lang: &Lang, timer_seconds: Option<u64>) -> io::Result<()> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("{}\n\n", lang.title)),
        ResetColor
    )?;
    draw_hangman(game.attempts_left(), game.max_attempts())?;
    execute!(
        stdout,
        Print(lang.word_label),
        SetForegroundColor(Color::Green),
        Print(game.display_word()),
        ResetColor,
        Print("\n\n"),
        Print(lang.guessed_label),
        SetForegroundColor(Color::Magenta),
        Print(game.display_guessed()),
        ResetColor,
        Print("\n\n"),
        Print(lang.attempts_label),
        SetForegroundColor(Color::Red),
        Print(game.attempts_left()),
        ResetColor,
    )?;
    if let Some(sec) = timer_seconds {
        execute!(
            stdout,
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print(format!("Time left: {}s", sec)),
            ResetColor,
        )?;
    }
    execute!(stdout, Print("\n\n"))?;
    Ok(())
}

/// Get a guess from the user via line input.
/// Returns Some(letter) if valid, None if user wants to quit.
fn get_guess(lang: &Lang) -> io::Result<Option<char>> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::White),
        Print(lang.prompt_guess),
        ResetColor
    )?;
    stdout.flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input == "salir" || input == "quit" || input == "sair" || input == "q" {
        return Ok(None);
    }
    // Find the first alphabetic character
    for c in input.chars() {
        if c.is_alphabetic() {
            return Ok(Some(c));
        }
    }
    Ok(Some('\0')) // invalid
}

fn get_guess_with_timeout(_lang: &Lang, timeout_seconds: u64) -> io::Result<GuessResult> {
    let mut stdout = io::stdout();
    let mut remaining = timeout_seconds;
    let mut input = String::new();
    // Print initial prompt
    let color = if remaining <= 3 { Color::Red } else if remaining <= 10 { Color::Yellow } else { Color::White };
    execute!(
        stdout,
        SetForegroundColor(color),
        Print(format!("Enter a letter (time left: {}s): ", remaining)),
        ResetColor,
    )?;
    stdout.flush()?;
    let start = Instant::now();
    loop {
        // Calculate remaining time
        let elapsed = start.elapsed().as_secs();
        if elapsed >= timeout_seconds {
            // Timeout
            execute!(stdout, Print("\n"))?;
            return Ok(GuessResult::Timeout);
        }
        let new_remaining = timeout_seconds - elapsed;
        if new_remaining != remaining {
            remaining = new_remaining;
            // Update prompt on same line
            let color = if remaining <= 3 { Color::Red } else if remaining <= 10 { Color::Yellow } else { Color::White };
            execute!(
                stdout,
                Print("\r"),
                Clear(ClearType::CurrentLine),
                SetForegroundColor(color),
                Print(format!("Enter a letter (time left: {}s): {}", remaining, input)),
                ResetColor,
            )?;
            stdout.flush()?;
        }
        // Poll for key event with 1 second timeout
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) if c.is_alphabetic() => {
                        input.push(c);
                        // Echo the character
                        execute!(stdout, Print(c))?;
                    }
                    KeyCode::Backspace => {
                        if input.pop().is_some() {
                            // Move cursor back, print space, move back again
                            execute!(
                                stdout,
                                Print("\x08 \x08"),
                            )?;
                        }
                    }
                    KeyCode::Enter => {
                        // Process input
                        execute!(stdout, Print("\n"))?;
                        if input.is_empty() {
                            // No input, treat as timeout? Let's treat as invalid and continue
                            continue;
                        }
                        // Check for quit words
                        let trimmed = input.trim().to_lowercase();
                        if trimmed == "salir" || trimmed == "quit" || trimmed == "sair" || trimmed == "q" {
                            return Ok(GuessResult::Quit);
                        }
                        // Find first alphabetic character
                        for ch in input.chars() {
                            if ch.is_alphabetic() {
                                return Ok(GuessResult::Letter(ch));
                            }
                        }
                        // No alphabetic character, clear input and continue
                        input.clear();
                        let color = if remaining <= 3 { Color::Red } else if remaining <= 10 { Color::Yellow } else { Color::White };
                        execute!(
                            stdout,
                            Print("\r"),
                            Clear(ClearType::CurrentLine),
                            SetForegroundColor(color),
                            Print(format!("Enter a letter (time left: {}s): ", remaining)),
                            ResetColor,
                        )?;
                        stdout.flush()?;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn play_game(mut game: Hangman, lang: &Lang) -> io::Result<()> {
    const TIMEOUT_SECONDS: u64 = 30;
    let raw_guard = terminal::enable_raw_mode();
    let raw = raw_guard.is_ok();
    let mut previous_attempts_left = game.attempts_left();
    loop {
        // Draw the game state (including hangman)
        draw_game_state(&game, lang, None)?;
        if game.is_won() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!("{}\n", lang.win_msg)),
                ResetColor,
                Print(lang.press_enter),
            )?;
            wait_for_enter()?;
            break;
        }
        if game.is_lost() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print(lang.lose_msg),
                SetForegroundColor(Color::Yellow),
                Print(game.word()),
                ResetColor,
                Print(format!("\n{}", lang.press_enter)),
            )?;
            wait_for_enter()?;
            break;
        }
        let guess_result = if raw {
            get_guess_with_timeout(lang, TIMEOUT_SECONDS)?
        } else {
            // Fallback to line input without timeout
            match get_guess(lang)? {
                Some(c) => GuessResult::Letter(c),
                None => GuessResult::Quit,
            }
        };
        match guess_result {
            GuessResult::Quit => break,
            GuessResult::Timeout => {
                // Lose an attempt
                let mut stdout = io::stdout();
                execute!(
                    stdout,
                    SetForegroundColor(Color::Yellow),
                    Print("Time's up! You lose an attempt.\n"),
                    ResetColor,
                )?;
                wait_for_enter()?;
                // Lose an attempt
                game.lose_attempt();
                let new_attempts_left = game.attempts_left();
                if new_attempts_left < previous_attempts_left {
                    draw_hangman_with_delay(
                        new_attempts_left,
                        game.max_attempts(),
                        previous_attempts_left,
                    )?;
                    previous_attempts_left = new_attempts_left;
                }
            }
            GuessResult::Letter(letter) => {
                match game.guess(letter) {
                    Ok(true) => {
                        // correct guess, continue
                    }
                    Ok(false) => {
                        // wrong guess: attempts decreased
                        // Check if a new body part should be drawn with delay
                        let new_attempts_left = game.attempts_left();
                        if new_attempts_left < previous_attempts_left {
                            // Redraw with delay
                            draw_hangman_with_delay(
                                new_attempts_left,
                                game.max_attempts(),
                                previous_attempts_left,
                            )?;
                            previous_attempts_left = new_attempts_left;
                        }
                    }
                    Err(err) => {
                        let msg = match err {
                            GuessError::NotLetter => lang.error_not_letter,
                            GuessError::AlreadyGuessed => lang.error_already_guessed,
                        };
                        let mut stdout = io::stdout();
                        execute!(
                            stdout,
                            SetForegroundColor(Color::Yellow),
                            Print(format!("{}\n", msg)),
                            ResetColor,
                        )?;
                        wait_for_enter()?;
                    }
                }
            }
        }
    }
    if raw {
        // raw_guard will disable raw mode when dropped
    }
    Ok(())
}

pub fn get_word_from_player(lang: &Lang) -> io::Result<String> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Yellow),
        Print(lang.word_input_warning),
        Print("\n\n"),
        ResetColor,
        SetForegroundColor(Color::Cyan),
        Print(lang.word_input_prompt),
        Print("\n"),
        ResetColor,
        Print(lang.word_input_instruction),
        Print("\n"),
    )?;
    stdout.flush()?;
    let mut word = String::new();
    io::stdin().read_line(&mut word)?;
    Ok(word.trim().to_string())
}

pub fn draw_menu(lang: &Lang) -> io::Result<()> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(format!("{}\n\n", lang.title)),
        ResetColor,
        Print(format!("{}\n", lang.menu_solo)),
        Print(format!("{}\n", lang.menu_multi)),
        Print(format!("{}\n\n", lang.menu_quit)),
        SetForegroundColor(Color::White),
        Print(lang.prompt_option),
        ResetColor,
    )?;
    stdout.flush()?;
    Ok(())
}

pub fn run_menu() -> io::Result<()> {
    let language = select_language()?;
    let lang = Lang::from_language(language);
    loop {
        draw_menu(&lang)?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        match choice {
            "1" => {
                let game = Hangman::random(lang.movies);
                play_game(game, &lang)?;
            }
            "2" => {
                let word = get_word_from_player(&lang)?;
                if word.is_empty() {
                    let game = Hangman::random(lang.movies);
                    play_game(game, &lang)?;
                } else {
                    let game = Hangman::new(&word, 6);
                    play_game(game, &lang)?;
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
