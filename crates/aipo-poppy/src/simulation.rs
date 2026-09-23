//! Headless deterministic game simulation engine for Poppy.

use crate::prng::PoppyRng;
use crate::world::World;

/// Headless simulation context executing fixed-rate game ticks.
#[derive(Debug)]
pub struct Simulation {
    /// ECS World containing entities and command buffer.
    world: World,
    /// Seeded PRNG.
    rng: PoppyRng,
    /// Current tick counter.
    tick: u64,
    /// Fixed delta time per tick in seconds (e.g. 1.0 / 60.0).
    fixed_dt: f64,
}

impl Simulation {
    /// Creates a new simulation with a fixed time-step and PRNG seed.
    #[must_use]
    pub fn new(seed: u64, fixed_dt: f64) -> Self {
        Self {
            world: World::new(),
            rng: PoppyRng::new(seed),
            tick: 0,
            fixed_dt,
        }
    }

    /// Access the world.
    #[must_use]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Access the world mutably.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Access the PRNG mutably.
    pub fn rng_mut(&mut self) -> &mut PoppyRng {
        &mut self.rng
    }

    /// Current tick number.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Fixed delta time.
    #[must_use]
    pub fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }

    /// Advances the simulation by one tick:
    /// 1. Integrates entity velocities.
    /// 2. Applies queued commands at the safe point.
    /// 3. Computes and returns the deterministic state digest.
    pub fn step(&mut self) -> u64 {
        self.tick += 1;
        self.world.integrate(self.fixed_dt);
        self.world.apply_deferred();
        self.world.digest(self.tick, self.rng.state())
    }

    /// Runs the simulation for `count` ticks, returning all state digests.
    pub fn run_ticks(&mut self, count: u64) -> Vec<u64> {
        let mut digests = Vec::with_capacity(count as usize);
        for _ in 0..count {
            digests.push(self.step());
        }
        digests
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_reproducibility() {
        let mut sim1 = Simulation::new(42, 1.0 / 60.0);
        let mut sim2 = Simulation::new(42, 1.0 / 60.0);

        sim1.world_mut()
            .spawn_immediate("player", 0.0, 0.0, 5.0, 5.0);
        sim2.world_mut()
            .spawn_immediate("player", 0.0, 0.0, 5.0, 5.0);

        let digests1 = sim1.run_ticks(60);
        let digests2 = sim2.run_ticks(60);

        assert_eq!(digests1, digests2, "simulations with same seed must match");
        assert_eq!(digests1.len(), 60);
    }
}
