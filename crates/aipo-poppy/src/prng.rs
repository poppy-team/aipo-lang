//! Deterministic pseudo-random number generator for the Poppy Game Engine.
//!
//! Simulation reproducibility is a primary design goal: with the same seed, every
//! calculation, branch and event in a game session must play out identically.

/// A deterministic xorshift64* pseudo-random number generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoppyRng {
    state: u64,
}

impl PoppyRng {
    /// Creates a new generator from a 64-bit seed.
    ///
    /// If seed is 0, defaults to 1 to prevent degenerate state.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    /// Current internal state.
    #[must_use]
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Generates the next raw `u64`.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Generates a float in the half-open interval `[0.0, 1.0)`.
    pub fn next_float(&mut self) -> f64 {
        const MANTISSA_BITS: u64 = 53;
        const MANTISSA_MASK: u64 = (1_u64 << MANTISSA_BITS) - 1;
        let bits = self.next_u64() & MANTISSA_MASK;
        (bits as f64) / ((1_u64 << MANTISSA_BITS) as f64)
    }

    /// Generates a pseudo-random integer in `[min, max]`.
    pub fn random_int(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let range = (max as i128 - min as i128 + 1) as u64;
        let offset = self.next_u64() % range;
        min + offset as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = PoppyRng::new(42);
        let mut rng2 = PoppyRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
            assert_eq!(rng1.next_float(), rng2.next_float());
            assert_eq!(rng1.random_int(-10, 10), rng2.random_int(-10, 10));
        }
    }

    #[test]
    fn test_rng_float_range() {
        let mut rng = PoppyRng::new(12345);
        for _ in 0..1000 {
            let f = rng.next_float();
            assert!((0.0..1.0).contains(&f), "float {f} outside [0.0, 1.0)");
        }
    }
}
