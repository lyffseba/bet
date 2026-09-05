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
        // For upper=3 the accept limit equals u64::MAX (since u64::MAX % 3 == 0),
        // so the only sample that would be rejected is u64::MAX itself.
        let mut rng = XorShift64::new(99);
        let first = rng.next_u64();
        if first != u64::MAX {
            let mut rng2 = XorShift64::new(99);
            assert_eq!(rng2.gen_range(3) as u64, first % 3);
        }
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn gen_range_rejection_path_uses_next_accepted_sample() {
        // upper = 2^63 + 1 => accept limit = 2^63 + 1, so ~50% of samples are
        // rejected. Pick a seed whose FIRST sample rejects; gen_range must then
        // return the first ACCEPTED sample reduced mod upper. A biased
        // `next_u64() % upper` implementation would fail this test.
        let upper = (1usize << 63) + 1;
        let limit = u64::MAX - (u64::MAX % upper as u64);
        let seed = (1u64..)
            .find(|&s| XorShift64::new(s).next_u64() >= limit)
            .expect("some seed must reject on the first sample");
        // Replay the stream: first sample rejects, then take the first accepted one.
        let mut seq = XorShift64::new(seed);
        let mut accepted = seq.next_u64();
        assert!(accepted >= limit, "seed selection guarantees first-sample rejection");
        while accepted >= limit {
            accepted = seq.next_u64();
        }
        let mut rng = XorShift64::new(seed);
        assert_eq!(rng.gen_range(upper) as u64, accepted % upper as u64);
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
        // Pin the golden fixture value (protocols/fixtures/wasm_goldens.json):
        // unbiased gen_range with seed 99 must select CHARLIE on every platform.
        assert_eq!(h.word(), "CHARLIE");
    }
}
