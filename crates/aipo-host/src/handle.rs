//! Generational handles for host-owned objects.
//!
//! Canon requires that a host object referenced by a script never uses freed state
//! (Fechamento Arquitetural §11, `docs/waves/wave-4-host-poppy.md`): a stale handle must
//! resolve to `none` or a `Failure` per the declared contract, and a security fault must
//! never surface as a Rust panic.
//!
//! A [`Handle`] is a slot index plus a generation, and a handle is only ever dereferenced
//! through the table that minted it. Releasing a slot bumps its generation, so a handle
//! from before the release addresses a generation that no longer occupies that slot and
//! resolves to stale — never to whatever the slot is reused for afterwards.

use crate::fault::HostFault;

/// A handle to a value owned by the host.
///
/// Opaque by design: a script can copy it, store it and pass it back, but the index and
/// generation are only meaningful to the [`HandleTable`] that minted it. The index is a
/// `usize` because the slot table is memory-bound: exhausting an index space that wide would
/// require holding that many slots, so the table needs no invented error path for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Handle {
    index: usize,
    generation: u32,
}

impl Handle {
    /// The slot index, for host-side diagnostics and tests.
    #[must_use]
    pub fn index(self) -> usize {
        self.index
    }

    /// The generation this handle was minted in.
    #[must_use]
    pub fn generation(self) -> u32 {
        self.generation
    }
}

impl std::fmt::Display for Handle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "handle({}:{})", self.index, self.generation)
    }
}

/// One slot: the generation currently occupying it and its value when live.
#[derive(Debug, Clone)]
struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

/// A generational table of host values.
///
/// The table owns every host object; scripts only ever hold [`Handle`] values. [`HandleTable::get`]
/// on a released or unknown handle yields `None`, and [`HandleTable::require`] turns that into
/// the stale-handle fault a non-nullable contract demands.
///
/// When a slot's generation space is exhausted the slot is left out of the reuse list instead
/// of wrapping, because a wrapped generation would let an ancient handle address the slot
/// again. Reaching that point takes 2^32 releases of one slot.
#[derive(Debug, Default)]
pub struct HandleTable<T> {
    /// Live and reusable slots, addressed by index. `None` marks a slot that must never be
    /// reused (generation exhausted).
    slots: Vec<Option<Slot<T>>>,
    /// Indices available for reuse, newest first.
    free: Vec<usize>,
    /// Number of live values.
    live: usize,
}

impl<T> HandleTable<T> {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }

    /// Stores a host value and returns its handle.
    ///
    /// Reuse takes a slot from the free list with the generation its release already bumped,
    /// so every handle minted before that release stays stale.
    pub fn insert(&mut self, value: T) -> Handle {
        let handle = match self.free.pop() {
            Some(index) => {
                let generation = self.slots[index].as_ref().map_or(1, |slot| slot.generation);
                self.slots[index] = Some(Slot {
                    generation,
                    value: Some(value),
                });
                Handle { index, generation }
            }
            None => {
                let index = self.slots.len();
                self.slots.push(Some(Slot {
                    generation: 1,
                    value: Some(value),
                }));
                Handle {
                    index,
                    generation: 1,
                }
            }
        };
        self.live += 1;
        handle
    }

    /// The value behind `handle`, or `None` when the handle is stale or unknown.
    ///
    /// Use this wherever the declared contract admits a stale handle resolving to `none`.
    #[must_use]
    pub fn get(&self, handle: Handle) -> Option<&T> {
        let slot = self.slots.get(handle.index)?.as_ref()?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_ref()
    }

    /// The value behind `handle` mutably, or `None` when the handle is stale or unknown.
    #[must_use]
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.index)?.as_mut()?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_mut()
    }

    /// Whether `handle` currently addresses a live value.
    #[must_use]
    pub fn contains(&self, handle: Handle) -> bool {
        self.get(handle).is_some()
    }

    /// The value behind `handle`, or the stale-handle fault.
    ///
    /// Callers use this where the declared contract says a handle is not optional; a
    /// contract that admits staleness uses [`HandleTable::get`] instead.
    ///
    /// # Errors
    ///
    /// [`HostFault::StaleHandle`] when the handle was released, belongs to another
    /// generation, or was never minted by this table.
    pub fn require(&self, handle: Handle) -> Result<&T, HostFault> {
        self.get(handle).ok_or(HostFault::StaleHandle { handle })
    }

    /// Takes the value out of `handle` and invalidates every handle minted for that slot.
    ///
    /// Returns `None` for a handle that is already stale, so a double release is harmless.
    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let slot = self.slots.get_mut(handle.index)?.as_mut()?;
        if slot.generation != handle.generation {
            return None;
        }
        let value = slot.value.take()?;
        self.live -= 1;

        // Bump before the slot is reusable: the next insert mints a different generation,
        // so `handle` can never resolve to the replacement value. An exhausted generation
        // retires the slot instead of wrapping.
        if slot.generation == u32::MAX {
            self.slots[handle.index] = None;
            return Some(value);
        }
        slot.generation += 1;
        self.free.push(handle.index);
        Some(value)
    }

    /// Number of live values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.live
    }

    /// Whether the table holds no live value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Iterates live handles and their values in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let slot = slot.as_ref()?;
            let value = slot.value.as_ref()?;
            Some((
                Handle {
                    index,
                    generation: slot.generation,
                },
                value,
            ))
        })
    }

    /// Drops every value and invalidates every handle.
    pub fn clear(&mut self) {
        for slot in self.slots.iter_mut().flatten() {
            if slot.generation == u32::MAX {
                continue;
            }
            slot.generation += 1;
            slot.value = None;
        }
        self.free.clear();
        self.live = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut table = HandleTable::new();
        let handle = table.insert("sword");
        assert_eq!(table.get(handle), Some(&"sword"));
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
    }

    #[test]
    fn test_released_handle_never_resolves_to_reused_slot() {
        let mut table = HandleTable::new();
        let first = table.insert(10);
        assert_eq!(table.remove(first), Some(10));
        // A double release is harmless and does not bump again.
        assert_eq!(table.remove(first), None);

        let second = table.insert(20);
        assert_eq!(second.index(), first.index());
        assert_ne!(second.generation(), first.generation());
        assert_eq!(
            table.get(first),
            None,
            "stale handle must not see the new value"
        );
        assert_eq!(table.get(second), Some(&20));
        assert!(!table.contains(first));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn test_stale_handle_is_a_fault_not_a_panic() {
        let mut table = HandleTable::new();
        let handle = table.insert(1);
        table.remove(handle);
        let fault = table.require(handle).expect_err("stale");
        assert_eq!(
            fault.code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_STALE_HANDLE
        );
    }

    #[test]
    fn test_unknown_handle_is_not_live() {
        let table: HandleTable<u8> = HandleTable::new();
        let unknown = Handle {
            index: 7,
            generation: 3,
        };

        assert_eq!(table.get(unknown), None);
        assert!(!table.contains(unknown));
        assert!(table.require(unknown).is_err());
    }

    #[test]
    fn test_generation_increases_across_reuse() {
        let mut table = HandleTable::new();
        let first = table.insert(1);
        table.remove(first);
        let second = table.insert(2);
        table.remove(second);
        let third = table.insert(3);
        assert!(third.generation() > second.generation());
        assert!(second.generation() > first.generation());
        assert_eq!(table.get(third), Some(&3));
    }

    #[test]
    fn test_get_mut_through_a_live_handle() {
        let mut table = HandleTable::new();
        let handle = table.insert(String::from("a"));
        table.get_mut(handle).expect("live").push('b');
        assert_eq!(table.get(handle).map(String::as_str), Some("ab"));
    }

    #[test]
    fn test_mutating_a_stale_handle_changes_nothing() {
        let mut table = HandleTable::new();
        let handle = table.insert(String::from("a"));
        table.remove(handle);
        let replacement = table.insert(String::from("b"));
        assert_eq!(replacement.index(), handle.index());
        assert!(table.get_mut(handle).is_none());
        assert_eq!(table.get(replacement).map(String::as_str), Some("b"));
    }

    #[test]
    fn test_clear_invalidates_every_handle() {
        let mut table = HandleTable::new();
        let handle = table.insert(1);
        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.get(handle), None);
    }

    #[test]
    fn test_iter_lists_live_values_only() {
        let mut table = HandleTable::new();
        let first = table.insert(1);
        let second = table.insert(2);
        table.remove(first);
        let live: Vec<_> = table
            .iter()
            .map(|(handle, value)| (handle, *value))
            .collect();
        assert_eq!(live, vec![(second, 2)]);
    }

    #[test]
    fn test_generation_exhaustion_retires_the_slot() {
        let mut table: HandleTable<u8> = HandleTable::new();
        let index = table.insert(0).index();
        // Move the slot to the last generation without looping 2^32 times.
        table.slots[index] = Some(Slot {
            generation: u32::MAX,
            value: Some(1),
        });
        let last = Handle {
            index,
            generation: u32::MAX,
        };
        assert_eq!(table.remove(last), Some(1));

        // The slot is not reused, so no handle can address a wrapped generation.
        assert!(table.is_empty());
        let fresh = table.insert(2);
        assert_ne!(fresh.index(), index);
        assert!(!table.contains(last));
    }
}
