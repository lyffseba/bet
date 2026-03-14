use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    execute,
    style::{Print, SetForegroundColor, Color, ResetColor},
    terminal::{self, Clear, ClearType},
};

use crate::game::Hangman;

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

pub fn draw_game_state(game: &Hangman) -> io::Result<()> {
    let mut stdout = io::stdout();
    clear_screen()?;
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("AHORCADO\n\n"),
        ResetColor
    )?;
    draw_hangman(game.attempts_left(), game.max_attempts())?;
    execute!(
        stdout,
        Print("Palabra: "),
        SetForegroundColor(Color::Green),
        Print(game.display_word()),
        ResetColor,
        Print("\n\nLetras adivinadas: "),
        SetForegroundColor(Color::Magenta),
        Print(game.display_guessed()),
        ResetColor,
        Print("\n\nIntentos restantes: "),
        SetForegroundColor(Color::Red),
        Print(game.attempts_left()),
        ResetColor,
        Print("\n\n")
    )?;
    Ok(())
}

/// Get a guess from the user via line input.
/// Returns Some(letter) if valid, None if user wants to quit.
fn get_guess() -> io::Result<Option<char>> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(Color::White),
        Print("Ingresa una letra (o 'salir' para salir): "),
        ResetColor
    )?;
    stdout.flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    if input == "salir" || input == "quit" || input == "q" {
        return Ok(None);
    }
    // Find the first alphabetic character
    for c in input.chars() {
        if c.is_ascii_alphabetic() {
            return Ok(Some(c));
        }
    }
    Ok(Some('\0')) // invalid
}

pub fn play_game(mut game: Hangman) -> io::Result<()> {
    let mut previous_attempts_left = game.attempts_left();
    loop {
        // Draw the game state (including hangman)
        draw_game_state(&game)?;
        if game.is_won() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print("¡Felicidades! ¡Ganaste!\n"),
                ResetColor,
                Print("Presiona Enter para continuar..."),
            )?;
            wait_for_enter()?;
            break;
        }
        if game.is_lost() {
            let mut stdout = io::stdout();
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print("¡Juego terminado! La palabra era: "),
                SetForegroundColor(Color::Yellow),
                Print(game.word()),
                ResetColor,
                Print("\nPresiona Enter para continuar..."),
            )?;
            wait_for_enter()?;
            break;
        }
        match get_guess()? {
            None => break, // quit
            Some('\0') => {
                // invalid input, ignore
            }
            Some(letter) => {
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
                    Err(msg) => {
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
        Print("Ingresa la palabra para que el otro jugador adivine (no se mostrará):\n"),
        ResetColor,
        Print("(Escribe la palabra y presiona Enter)\n"),
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
        Print("AHORCADO\n\n"),
        ResetColor,
        Print("1. Solo (película aleatoria)\n"),
        Print("2. Multijugador (un jugador pone la palabra)\n"),
        Print("3. Salir\n\n"),
        SetForegroundColor(Color::White),
        Print("Elige una opción: "),
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
