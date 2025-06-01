use crate::game::rng::Prng;
use bevy::prelude::*;
use bevy_rand::prelude::GlobalEntropy;
use rand::Rng;
use std::f32::consts::PI;

/// Samples a uniformly random point on the unit sphere (radius = 1.0),
/// returning [`Vec3`].
pub fn sample_point_on_sphere<R: Rng>(rng: &mut R) -> Vec3 {
    // Pick z ∈ [-1, 1] uniformly, and θ ∈ [0, 2π) uniformly.
    let z: f32 = rng.random_range(-1.0..1.0);
    let theta: f32 = rng.random_range(0.0..(2.0 * PI));

    // Radius in the xy‐plane so that x² + y² + z² = 1
    let r_xy = (1.0 - z * z).sqrt();

    let x = r_xy * theta.cos();
    let y = r_xy * theta.sin();

    Vec3::new(x, y, z)
}

pub trait RandomSpherePoint {
    fn random_sphere_point(&mut self, radius: f32) -> Vec3;
}

impl RandomSpherePoint for GlobalEntropy<'_, Prng> {
    fn random_sphere_point(&mut self, radius: f32) -> Vec3 {
        sample_point_on_sphere(self) * radius
    }
}
