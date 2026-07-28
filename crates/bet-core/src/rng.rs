//! Tiny deterministic PRNG (xorshift64*) for seeded game randomness.
//!
//! Cross-platform contract: the same seed must produce the same stream on
//! wasm32 and native 64-bit hosts.

/// Seeded xorshift64* — fast, portable, good enough for games.
#[derive(Debug, Clone, Copy)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        // Avoid zero state which collapses the generator.
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
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
    /// - Uses full `u64` samples (never cast to `usize` before reduce) so
    ///   wasm32 matches native.
    /// - Rejection sampling removes modulo bias when `upper` is not a
    ///   power of two.
    pub fn gen_range(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        let upper = upper as u64;
        // Largest multiple of `upper` that fits in u64.
        let limit = u64::MAX - (u64::MAX % upper);
        loop {
            let r = self.next_u64();
            if r < limit {
                return (r % upper) as usize;
            }
        }
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
    fn gen_range_matches_full_u64_when_no_reject() {
        // For upper=3, almost all samples are accepted; first sample matches % 3.
        let mut rng = XorShift64::new(99);
        let first = rng.next_u64();
        let limit = u64::MAX - (u64::MAX % 3);
        if first < limit {
            let mut rng2 = XorShift64::new(99);
            assert_eq!(rng2.gen_range(3) as u64, first % 3);
        }
    }

    #[test]
    fn gen_range_stream_is_stable() {
        // Pin a short stream so wasm + native + fixture stay aligned.
        let mut rng = XorShift64::new(1);
        let got: Vec<usize> = (0..12).map(|_| rng.gen_range(5)).collect();
        assert_eq!(got, vec![0, 2, 3, 3, 3, 1, 3, 1, 4, 0, 3, 2]);
    }

    #[test]
    fn hangman_seed_99_word_stable() {
        use crate::hangman::Hangman;
        let words = ["ALPHA", "BRAVO", "CHARLIE"];
        let h = Hangman::from_seed(99, &words, 6);
        // After unbiased gen_range, word must stay pinned in fixture.
        assert_eq!(h.word(), Hangman::from_seed(99, &words, 6).word());
    }
}
