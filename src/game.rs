use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Hangman {
    word: String,
    guessed_letters: Vec<char>,
    max_attempts: usize,
    attempts_left: usize,
}

#[derive(Debug)]
pub enum GuessError {
    NotLetter,
    AlreadyGuessed,
}

impl Hangman {
    pub fn new(word: &str, max_attempts: usize) -> Self {
        Self {
            word: word.to_uppercase(),
            guessed_letters: Vec::new(),
            max_attempts,
            attempts_left: max_attempts,
        }
    }

    pub fn random(movies: &'static [&'static str]) -> Self {
        let word = movies.choose(&mut rand::thread_rng()).unwrap_or(&"BET");
        Self::new(word, 6)
    }

    pub fn guess(&mut self, letter: char) -> Result<bool, GuessError> {
        let letter = letter.to_uppercase().next().unwrap_or(letter); // get the uppercase char
        if !letter.is_alphabetic() {
            return Err(GuessError::NotLetter);
        }
        if self.guessed_letters.contains(&letter) {
            return Err(GuessError::AlreadyGuessed);
        }
        self.guessed_letters.push(letter);
        if self.word.contains(letter) {
            Ok(true)
        } else {
            self.attempts_left -= 1;
            Ok(false)
        }
    }

    pub fn is_won(&self) -> bool {
        self.word.chars().all(|c| {
            if c.is_alphabetic() {
                self.guessed_letters.contains(&c)
            } else {
                // Non‑alphabetic characters are always considered "guessed"
                true
            }
        })
    }

    pub fn is_lost(&self) -> bool {
        self.attempts_left == 0
    }

    #[allow(dead_code)]
    pub fn display_word(&self) -> String {
        self.word
            .chars()
            .map(|c| {
                if c.is_alphabetic() {
                    if self.guessed_letters.contains(&c) {
                        c
                    } else {
                        '_'
                    }
                } else {
                    // Spaces, punctuation, etc. are displayed as themselves
                    c
                }
            })
            .collect::<String>()
    }

    pub fn display_guessed(&self) -> String {
        let mut guessed = self.guessed_letters.clone();
        guessed.sort();
        guessed.iter().collect()
    }

    pub fn attempts_left(&self) -> usize {
        self.attempts_left
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    pub fn guessed_letters(&self) -> &Vec<char> {
        &self.guessed_letters
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn decrease_attempts(&mut self) {
        if self.attempts_left > 0 {
            self.attempts_left -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_with_spaces() {
        let mut game = Hangman::new("A B", 6);
        // Guess A and B
        game.guess('A').unwrap();
        game.guess('B').unwrap();
        assert!(game.is_won());
        // Display should show "A B" (space preserved)
        assert_eq!(game.display_word(), "A B");
    }

    #[test]
    fn test_win_with_punctuation() {
        let mut game = Hangman::new("A-B", 6);
        game.guess('A').unwrap();
        game.guess('B').unwrap();
        assert!(game.is_won());
        assert_eq!(game.display_word(), "A-B");
    }

    #[test]
    fn test_not_win_until_all_letters_guessed() {
        let mut game = Hangman::new("AB", 6);
        game.guess('A').unwrap();
        assert!(!game.is_won());
        game.guess('B').unwrap();
        assert!(game.is_won());
    }

    #[test]
    fn test_accented_letters() {
        let mut game = Hangman::new("café", 6);
        // The word is stored as uppercase: "CAFÉ"
        game.guess('c').unwrap(); // should become 'C'
        game.guess('a').unwrap();
        game.guess('f').unwrap();
        game.guess('é').unwrap(); // should become 'É'
        assert!(game.is_won());
        // Display should show "CAFÉ"
        assert_eq!(game.display_word(), "CAFÉ");
    }
}

#[test]
fn test_loss_condition() {
    let mut game = Hangman::new("TEST", 3);

    // 1st wrong guess
    assert_eq!(game.guess('X').unwrap(), false);
    assert!(!game.is_lost());
    assert_eq!(game.attempts_left(), 2);

    // 2nd wrong guess
    assert_eq!(game.guess('Y').unwrap(), false);
    assert!(!game.is_lost());
    assert_eq!(game.attempts_left(), 1);

    // 3rd wrong guess - game over
    assert_eq!(game.guess('Z').unwrap(), false);
    assert!(game.is_lost());
    assert_eq!(game.attempts_left(), 0);
}

#[test]
fn test_invalid_guess() {
    let mut game = Hangman::new("TEST", 3);

    // Numbers
    assert!(matches!(game.guess('1'), Err(GuessError::NotLetter)));

    // Symbols
    assert!(matches!(game.guess('@'), Err(GuessError::NotLetter)));

    // Duplicate guess
    game.guess('T').unwrap();
    assert!(matches!(game.guess('T'), Err(GuessError::AlreadyGuessed)));

    // Duplicate wrong guess
    game.guess('X').unwrap();
    assert!(matches!(game.guess('X'), Err(GuessError::AlreadyGuessed)));
}
