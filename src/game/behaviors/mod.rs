pub mod ball_counter;
pub mod camera;
pub mod dead;
pub mod despawn;
pub mod dynamic_character_controller;
pub mod end_game;
pub mod enemy_controller;
pub mod grounded;
pub mod knocked_over;
pub mod restore_data;
pub mod spawn;
pub mod stopwatch;
pub mod stun;
pub mod target_ent;
pub mod zap_stuns;

use crate::game::utils::extensions::vec2::Vec2Ext;
use crate::game::utils::extensions::vec3::Vec3Ext;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct MaxMovementSpeed(pub f32);

pub fn clamp_velocity_to_max_xz(orig_vel: LinearVelocity, max_speed: f32) -> LinearVelocity {
    let mut new_vel: Vec2 = orig_vel.0.to_vec2();
    let current_speed_squared = new_vel.length_squared();
    let max_speed_squared = max_speed * max_speed;

    if current_speed_squared > max_speed_squared && current_speed_squared > 0.0 {
        new_vel = new_vel.normalize() * max_speed;
    }

    LinearVelocity(new_vel.to_vec3() + Vec3::Y * orig_vel.0.y)
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(ball_counter::plugin);
    app.add_plugins(camera::plugin);
    app.add_plugins(dynamic_character_controller::plugin);
    app.add_plugins(dead::plugin);
    app.add_plugins(despawn::plugin);
    app.add_plugins(end_game::plugin);
    app.add_plugins(enemy_controller::plugin);
    app.add_plugins(grounded::plugin);
    app.add_plugins(knocked_over::plugin);
    app.add_plugins(spawn::plugin);
    app.add_plugins(stopwatch::plugin);
    app.add_plugins(stun::plugin);
    app.add_plugins(target_ent::plugin);
    app.add_plugins(restore_data::plugin);
    app.add_plugins(zap_stuns::plugin);
}
