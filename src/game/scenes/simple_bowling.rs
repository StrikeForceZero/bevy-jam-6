use crate::game::utils::extensions::vec2::Vec2Ext;

use crate::game::camera::CameraTarget;
use crate::game::prefabs::bowling_ball::{BOWLING_BALL_RADIUS, BowlingBall};
use crate::game::prefabs::bowling_pin::{BowlingPin, PIN_WIDTH};
use crate::game::screens::Screen;
use avian3d::prelude::{
    Collider, ExternalAngularImpulse, ExternalImpulse, Friction, Mass, Restitution, RigidBody,
    Sensor,
};
use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[require(Visibility)]
pub struct LevelRoot;

#[auto_plugin(app=_app)]
pub fn plugin(_app: &mut App) {}