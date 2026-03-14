use crate::wordlist::MOVIES;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct Hangman {
    word: String,
    guessed_letters: Vec<char>,
    max_attempts: usize,
    attempts_left: usize,
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

    pub fn random() -> Self {
        let word = MOVIES.choose(&mut rand::thread_rng()).unwrap();
        Self::new(word, 6)
    }

    pub fn guess(&mut self, letter: char) -> Result<bool, &str> {
        let letter = letter.to_ascii_uppercase();
        if !letter.is_ascii_alphabetic() {
            return Err("Please enter a letter A-Z");
        }
        if self.guessed_letters.contains(&letter) {
            return Err("You already guessed that letter");
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
        self.word
            .chars()
            .all(|c| self.guessed_letters.contains(&c))
    }

    pub fn is_lost(&self) -> bool {
        self.attempts_left == 0
    }

    pub fn display_word(&self) -> String {
        self.word
            .chars()
            .map(|c| if self.guessed_letters.contains(&c) { c } else { '_' })
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

    pub fn word(&self) -> &str {
        &self.word
    }
}
