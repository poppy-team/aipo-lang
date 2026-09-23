//! Deferred ECS command buffer for safe-point mutations.
//!
//! Fechamento Arquitetural §8 requires that structural changes (spawn, despawn, component
//! additions/removals) enter a command buffer and apply strictly at safe points defined by
//! the scheduler. This prevents iterator invalidation during query traversals and guarantees
//! deterministic execution ordering.

use aipo_host::Handle;

/// A deferred command recorded during system execution.
#[derive(Debug, Clone, PartialEq)]
pub enum PoppyCommand {
    /// Spawns an entity with an allocated handle, tag, and initial transform/velocity.
    Spawn {
        /// Entity generational handle.
        handle: Handle,
        /// Entity tag / group name.
        tag: String,
        /// Initial x position.
        x: f64,
        /// Initial y position.
        y: f64,
        /// Initial x velocity.
        vx: f64,
        /// Initial y velocity.
        vy: f64,
    },
    /// Despawns an entity at the safe point.
    Despawn {
        /// Entity generational handle.
        handle: Handle,
    },
    /// Updates position at the safe point.
    SetPosition {
        /// Entity generational handle.
        handle: Handle,
        /// New x coordinate.
        x: f64,
        /// New y coordinate.
        y: f64,
    },
    /// Updates velocity at the safe point.
    SetVelocity {
        /// Entity generational handle.
        handle: Handle,
        /// New x velocity.
        vx: f64,
        /// New y velocity.
        vy: f64,
    },
}

/// A FIFO queue of commands accumulated between safe points.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CommandBuffer {
    commands: Vec<PoppyCommand>,
}

impl CommandBuffer {
    /// Creates a new, empty command buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a command to the buffer.
    pub fn push(&mut self, cmd: PoppyCommand) {
        self.commands.push(cmd);
    }

    /// Whether the buffer has no pending commands.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Number of queued commands.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Drains all commands, resetting the buffer.
    pub fn drain(&mut self) -> Vec<PoppyCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Clears the buffer without returning commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_host::HandleTable;

    #[test]
    fn test_command_buffer_queueing() {
        let mut table: HandleTable<i32> = HandleTable::new();
        let h1 = table.insert(1);
        let h2 = table.insert(2);

        let mut buf = CommandBuffer::new();
        assert!(buf.is_empty());

        buf.push(PoppyCommand::Despawn { handle: h1 });
        buf.push(PoppyCommand::SetPosition {
            handle: h2,
            x: 10.0,
            y: 20.0,
        });

        assert_eq!(buf.len(), 2);
        let drained = buf.drain();
        assert_eq!(drained.len(), 2);
        assert!(buf.is_empty());
    }
}
