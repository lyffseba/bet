//! Simple FNV-1a 64-bit fingerprint for golden state hashes.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x100_0000_01b3;

#[derive(Debug, Clone, Copy)]
pub struct Hasher {
    state: u64,
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    pub fn new() -> Self {
        Self { state: FNV_OFFSET }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.state ^= u64::from(v);
        self.state = self.state.wrapping_mul(FNV_PRIME);
    }

    pub fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u8(b);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn finish(self) -> u64 {
        self.state
    }
}

/// Hex-encoded 16-char fingerprint (stable for fixtures).
pub fn fingerprint_hex(h: u64) -> String {
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_empty() {
        assert_eq!(Hasher::new().finish(), FNV_OFFSET);
    }

    #[test]
    fn known_bytes() {
        let mut h = Hasher::new();
        h.write_str("bet");
        let a = h.finish();
        let mut h2 = Hasher::new();
        h2.write_str("bet");
        assert_eq!(a, h2.finish());
    }
}
