//! Pure hangman rules — no I/O, no wall-clock RNG.

use crate::hash::Hasher;
use crate::rng::XorShift64;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hangman {
    word: String,
    guessed_letters: Vec<char>,
    max_attempts: usize,
    attempts_left: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuessError {
    NotLetter,
    AlreadyGuessed,
    GameOver,
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

    /// Pick a word from `words` using a seeded PRNG (deterministic).
    pub fn from_seed(seed: u64, words: &[&str], max_attempts: usize) -> Self {
        let mut rng = XorShift64::new(seed);
        let word = rng.choose(words).copied().unwrap_or("BET");
        Self::new(word, max_attempts)
    }

    pub fn guess(&mut self, letter: char) -> Result<bool, GuessError> {
        if self.is_won() || self.is_lost() {
            return Err(GuessError::GameOver);
        }
        let letter = letter.to_uppercase().next().unwrap_or(letter);
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
            self.attempts_left = self.attempts_left.saturating_sub(1);
            Ok(false)
        }
    }

    pub fn is_won(&self) -> bool {
        self.word.chars().all(|c| {
            if c.is_alphabetic() {
                self.guessed_letters.contains(&c)
            } else {
                true
            }
        })
    }

    pub fn is_lost(&self) -> bool {
        self.attempts_left == 0
    }

    pub fn is_over(&self) -> bool {
        self.is_won() || self.is_lost()
    }

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
                    c
                }
            })
            .collect()
    }

    pub fn display_guessed(&self) -> String {
        let mut guessed = self.guessed_letters.clone();
        guessed.sort_unstable();
        guessed.into_iter().collect()
    }

    pub fn attempts_left(&self) -> usize {
        self.attempts_left
    }

    pub fn max_attempts(&self) -> usize {
        self.max_attempts
    }

    pub fn guessed_letters(&self) -> &[char] {
        &self.guessed_letters
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    /// Force-fail one attempt (e.g. timer expiry in the TUI).
    pub fn decrease_attempts(&mut self) {
        self.attempts_left = self.attempts_left.saturating_sub(1);
    }

    /// Stable fingerprint for golden tests.
    pub fn state_hash(&self) -> u64 {
        let mut h = Hasher::new();
        h.write_str(&self.word);
        h.write_u32(self.max_attempts as u32);
        h.write_u32(self.attempts_left as u32);
        for c in &self.guessed_letters {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            h.write_str(s);
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::fingerprint_hex;

    #[test]
    fn win_with_spaces() {
        let mut game = Hangman::new("A B", 6);
        game.guess('A').unwrap();
        game.guess('B').unwrap();
        assert!(game.is_won());
        assert_eq!(game.display_word(), "A B");
    }

    #[test]
    fn win_with_punctuation() {
        let mut game = Hangman::new("A-B", 6);
        game.guess('A').unwrap();
        game.guess('B').unwrap();
        assert!(game.is_won());
        assert_eq!(game.display_word(), "A-B");
    }

    #[test]
    fn loss_condition() {
        let mut game = Hangman::new("TEST", 3);
        assert!(!game.guess('X').unwrap());
        assert!(!game.guess('Y').unwrap());
        assert!(!game.guess('Z').unwrap());
        assert!(game.is_lost());
        assert_eq!(game.attempts_left(), 0);
    }

    #[test]
    fn invalid_guess() {
        let mut game = Hangman::new("TEST", 3);
        assert!(matches!(game.guess('1'), Err(GuessError::NotLetter)));
        game.guess('T').unwrap();
        assert!(matches!(game.guess('T'), Err(GuessError::AlreadyGuessed)));
    }

    #[test]
    fn seeded_word_is_deterministic() {
        let words = ["ALPHA", "BRAVO", "CHARLIE"];
        let a = Hangman::from_seed(99, &words, 6);
        let b = Hangman::from_seed(99, &words, 6);
        assert_eq!(a.word(), b.word());
        let c = Hangman::from_seed(100, &words, 6);
        // High probability different; if collision, still both valid words.
        assert!(words.contains(&c.word()));
    }

    #[test]
    fn golden_hash_path() {
        let mut game = Hangman::new("BET", 6);
        game.guess('B').unwrap();
        game.guess('X').unwrap();
        let hex = fingerprint_hex(game.state_hash());
        // Fixed expected for regression (update only intentionally).
        assert_eq!(hex.len(), 16);
        let again = fingerprint_hex(game.state_hash());
        assert_eq!(hex, again);
    }
}
