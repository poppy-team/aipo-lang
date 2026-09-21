//! Deterministic xorshift64* PRNG shared by every generative harness.
//!
//! The generator is intentionally tiny and dependency-free so seeds stay stable
//! across toolchains: the same seed always yields the same sequence.

/// Deterministic PRNG; `seed.max(1)` avoids the degenerate zero state.
pub struct Rng(pub u64);

impl Rng {
    /// Creates a generator from an explicit seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    /// Next raw `u64` (xorshift64*).
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform value in `0..bound` (`0` when `bound == 0`).
    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        let value = (self.next_u64() % bound as u64) as usize;
        value
    }

    /// One choice from a non-empty slice.
    pub fn choose<'a, T>(&mut self, options: &'a [T]) -> &'a T {
        &options[self.below(options.len())]
    }

    /// Random byte.
    pub fn byte(&mut self) -> u8 {
        #[allow(clippy::cast_possible_truncation)]
        let value = (self.next_u64() & 0xFF) as u8;
        value
    }

    /// Random `true` with probability `1/n`.
    pub fn one_in(&mut self, n: usize) -> bool {
        self.below(n) == 0
    }
}
