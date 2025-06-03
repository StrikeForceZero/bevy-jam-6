use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{
    behaviors::move_towards::MoveTowards,
    prefabs::{spawner::Spawner, tower::Tower},
    scenes::game::LevelRoot,
};

fn spawn(
    mut commands: Commands,
    time: Res<Time>,
    level_ent_q: Single<Entity, With<LevelRoot>>,
    tower_ent_q: Single<Entity, With<Tower>>,
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
                trans.with_scale(Vec3::splat(15.)),
                MoveTowards(tower_ent),
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
