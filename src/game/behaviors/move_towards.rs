use super::MovementSpeed;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target=Attracting)]
pub struct MoveTowards(pub Entity);

#[auto_register_type]
#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship=MoveTowards)]
pub struct Attracting(Vec<Entity>);

fn move_to_target(
    mut enemies: Query<(
        &mut LinearVelocity,
        &GlobalTransform,
        &MovementSpeed,
        &MoveTowards,
    )>,
    attractors: Query<&GlobalTransform, With<Attracting>>,
) {
    for (mut vel, tf_enemy, speed, towards) in enemies.iter_mut() {
        let tf_atractor = attractors.get(towards.0).expect("relationship");

        let dir = tf_atractor.translation().xz() - tf_enemy.translation().xz();
        let vel2 = Dir2::new(dir).map_or(Vec2::ZERO, |d| d * speed.0);
        vel.0 = Vec3::new(vel2.x, 0.0, vel2.y);
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, move_to_target); // TODO only set once
}
