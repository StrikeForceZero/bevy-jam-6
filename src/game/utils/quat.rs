use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

pub fn get_pitch_and_roll(quat: Quat) -> (f32, f32) {
    // Local forward and right vectors
    let local_forward = Vec3::Z;
    let local_right = Vec3::X;

    // Transform to world space
    let world_forward = quat * local_forward;
    let world_right = quat * local_right;

    // Pitch: angle between forward vector and horizontal plane (XZ)
    let pitch = world_forward
        .y
        .atan2((world_forward.x.powi(2) + world_forward.z.powi(2)).sqrt());

    // Roll: angle between right vector and horizontal plane (YZ)
    let roll = world_right
        .y
        .atan2((world_right.x.powi(2) + world_right.z.powi(2)).sqrt());

    (pitch, roll)
}
