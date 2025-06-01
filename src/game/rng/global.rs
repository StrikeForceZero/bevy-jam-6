use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::rng::{Prng, Seed};
use bevy_prng::WyRand;
use bevy_rand::prelude::{GlobalEntropy, GlobalRngEntity, RngSeed};
use rand::SeedableRng;

#[derive(SystemParam)]
pub struct GlobalRng<'w, 's> {
    #[allow(dead_code)]
    pub rng: GlobalEntropy<'w, Prng>,
    #[allow(dead_code)]
    pub global: GlobalRngEntity<'w, 's, Prng>,
}

#[allow(dead_code)]
impl<'w> GlobalRng<'w, '_> {
    pub fn rng(&mut self) -> &mut GlobalEntropy<'w, Prng> {
        &mut self.rng
    }
    pub fn seed(&self) -> &RngSeed<Prng> {
        self.global.seed()
    }
    pub fn seed_bytes(&self) -> Seed {
        let seed = self.global.clone_seed();
        let seed_bytes: <WyRand as SeedableRng>::Seed = seed;
        seed_bytes
    }
    pub fn reseed(&mut self, seed_bytes: Seed) {
        self.global.rng_commands().reseed(seed_bytes);
    }
}

#[auto_plugin(app=_app)]
pub(crate) fn plugin(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rng::{RngPlugin, SEED_LEN, ZERO_SEED};
    use bevy::ecs::system::RunSystemOnce;
    use rand::RngCore;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RngPlugin);
        app
    }

    fn get_seed_system(rng: GlobalRng) -> Seed {
        rng.seed_bytes()
    }

    fn set_seed_system(In(seed): In<Seed>, mut rng: GlobalRng) {
        rng.reseed(seed);
    }

    fn next_u32_system(mut rng: GlobalRng) -> u32 {
        rng.rng.next_u32()
    }

    fn set_seed(app: &mut App, seed: Seed) {
        app.world_mut()
            .run_system_once_with(set_seed_system, seed)
            .unwrap()
    }

    fn get_seed(app: &mut App) -> Seed {
        app.world_mut().run_system_once(get_seed_system).unwrap()
    }

    fn next_u32(app: &mut App) -> u32 {
        app.world_mut().run_system_once(next_u32_system).unwrap()
    }

    #[test]
    fn deterministic() {
        let mut app = test_app();
        app.update();
        set_seed(&mut app, ZERO_SEED);
        assert_eq!(next_u32(&mut app), 2371481814);
        assert_eq!(next_u32(&mut app), 412509173);
        set_seed(&mut app, ZERO_SEED);
        assert_eq!(next_u32(&mut app), 2371481814);
        assert_eq!(next_u32(&mut app), 412509173);
    }

    #[test]
    fn get_set_seed() {
        let mut app = test_app();
        set_seed(&mut app, ZERO_SEED);
        assert_eq!(get_seed(&mut app), ZERO_SEED);
        set_seed(&mut app, [1; SEED_LEN]);
        assert_eq!(get_seed(&mut app), [1; SEED_LEN]);
    }
}
