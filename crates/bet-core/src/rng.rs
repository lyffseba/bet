//! Tiny deterministic PRNG (xorshift64*) for seeded game randomness.

/// Seeded xorshift64* — fast, no_std-friendly, good enough for games.
#[derive(Debug, Clone, Copy)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        // Avoid zero state which collapses the generator.
        Self {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..upper` (upper must be > 0).
    ///
    /// Uses full `u64` modulus so results match across 32-bit (wasm32) and
    /// 64-bit hosts. Never cast `next_u64()` to `usize` before `%`.
    pub fn gen_range(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        let upper = upper as u64;
        (self.next_u64() % upper) as usize
    }

    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.gen_range(items.len())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = XorShift64::new(42);
        let mut b = XorShift64::new(42);
        for _ in 0..32 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = XorShift64::new(1);
        let mut b = XorShift64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn gen_range_uses_full_u64_modulus() {
        // Fixed stream: first next_u64 after seed 99 must reduce mod 3 stably
        // on both wasm32 and native (regression for cast-to-usize bug).
        let mut rng = XorShift64::new(99);
        let first = rng.next_u64();
        let mut rng2 = XorShift64::new(99);
        let idx = rng2.gen_range(3);
        assert_eq!(idx as u64, first % 3);
    }

    #[test]
    fn hangman_seed_99_picks_charlie() {
        use crate::hangman::Hangman;
        let words = ["ALPHA", "BRAVO", "CHARLIE"];
        let h = Hangman::from_seed(99, &words, 6);
        assert_eq!(h.word(), "CHARLIE");
    }
}
