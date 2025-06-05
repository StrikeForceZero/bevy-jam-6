use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{prefabs::spawner::Spawner, scenes::game::LevelRoot};

use super::target_ent::TargetEnt;

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    level_ent_q: Single<Entity, With<LevelRoot>>,
    // TODO: add mar
    tower_ent_q: Single<Entity /*, With<T>*/>,
    mut spawners: Query<(&mut Spawner, &Transform)>,
) {
    let level_ent = level_ent_q.into_inner();
    let tower_ent = tower_ent_q.into_inner();
    for (mut spawner, trans) in spawners.iter_mut() {
        if spawner.spawn_left == 0 {
            return;
        }
        if spawner.time_to_next_spawn.is_zero() {
            commands.entity(level_ent).with_child((
                Name::new("Skele"),
                spawner.spawns,
                // TODO scale should be set in the enemy spawner
                trans.with_scale(Vec3::splat(15.)),
                TargetEnt {
                    target_ent: tower_ent,
                    // TODO migrate to using colliders for determining proximity
                    within_distance: 20.0,
                },
            ));

            spawner.spawn_left -= 1;
            spawner.time_to_next_spawn = spawner.spawn_duration;
        }
        spawner.time_to_next_spawn = spawner.time_to_next_spawn.saturating_sub(time.delta());
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, spawn);
}
