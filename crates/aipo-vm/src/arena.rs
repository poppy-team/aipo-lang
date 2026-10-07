//! Safe linear bump arena allocator with hard quota and O(1) bulk resets (Marco 2 / ADP-014).

use std::fmt;

/// Errors occurring during arena allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArenaError {
    /// Allocation request exceeded the arena's maximum capacity.
    OutOfMemory {
        /// Requested size in bytes.
        requested: usize,
        /// Remaining capacity in bytes.
        available: usize,
    },
}

impl fmt::Display for ArenaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory {
                requested,
                available,
            } => {
                write!(
                    f,
                    "arena out of memory: requested {requested} bytes, but only {available} bytes available"
                )
            }
        }
    }
}

impl std::error::Error for ArenaError {}

/// Linear bump allocator operating over a contiguous byte buffer with a hard quota.
#[derive(Debug, Clone)]
pub struct ArenaAllocator {
    buffer: Vec<u8>,
    capacity: usize,
    peak: usize,
}

impl ArenaAllocator {
    /// Default arena capacity (32 KiB).
    pub const DEFAULT_CAPACITY: usize = 32 * 1024;

    /// Constructs an arena with a specific maximum byte capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            peak: 0,
        }
    }

    /// Constructs an arena with the default capacity (32 KiB).
    #[must_use]
    pub fn with_default_capacity() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }

    /// Allocates a byte slice into the arena, copying `bytes`.
    ///
    /// # Errors
    /// Returns [`ArenaError::OutOfMemory`] if the allocation exceeds the capacity quota.
    pub fn alloc_bytes(&mut self, bytes: &[u8]) -> Result<&[u8], ArenaError> {
        let len = bytes.len();
        if self.buffer.len() + len > self.capacity {
            return Err(ArenaError::OutOfMemory {
                requested: len,
                available: self.capacity.saturating_sub(self.buffer.len()),
            });
        }
        let start = self.buffer.len();
        self.buffer.extend_from_slice(bytes);
        if self.buffer.len() > self.peak {
            self.peak = self.buffer.len();
        }
        Ok(&self.buffer[start..start + len])
    }

    /// Allocates an immutable UTF-8 string into the arena.
    ///
    /// # Errors
    /// Returns [`ArenaError::OutOfMemory`] if the allocation exceeds capacity.
    pub fn alloc_str(&mut self, s: &str) -> Result<&str, ArenaError> {
        let bytes = self.alloc_bytes(s.as_bytes())?;
        std::str::from_utf8(bytes).map_err(|_| ArenaError::OutOfMemory {
            requested: 0,
            available: 0,
        })
    }

    /// Resets the arena in O(1) time, clearing all allocated data.
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// Total configured capacity in bytes.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of bytes currently allocated in the arena.
    #[must_use]
    pub fn used(&self) -> usize {
        self.buffer.len()
    }

    /// Number of bytes available before reaching the hard quota.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.capacity.saturating_sub(self.buffer.len())
    }

    /// Peak byte usage observed since creation.
    #[must_use]
    pub fn peak(&self) -> usize {
        self.peak
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocation_and_o1_reset() {
        let mut arena = ArenaAllocator::new(100);
        assert_eq!(arena.capacity(), 100);
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 100);

        let slice = arena.alloc_bytes(b"hello").unwrap();
        assert_eq!(slice, b"hello");
        assert_eq!(arena.used(), 5);
        assert_eq!(arena.remaining(), 95);

        let s = arena.alloc_str(" world").unwrap();
        assert_eq!(s, " world");
        assert_eq!(arena.used(), 11);
        assert_eq!(arena.peak(), 11);

        // Reset in O(1)
        arena.reset();
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 100);
        assert_eq!(arena.peak(), 11); // Peak is preserved across resets
    }

    #[test]
    fn test_arena_hard_quota_out_of_memory() {
        let mut arena = ArenaAllocator::new(10);
        assert!(arena.alloc_bytes(&[1; 8]).is_ok());
        let err = arena.alloc_bytes(&[2; 4]).unwrap_err();
        assert_eq!(
            err,
            ArenaError::OutOfMemory {
                requested: 4,
                available: 2,
            }
        );
    }
}
