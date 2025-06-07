use crate::game::behaviors::stun::Stunned;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
/// Despawn At [`Res<Time>`].elapsed_secs() >= target
pub struct DespawnAt(pub f32);

fn despawn_expired(
    mut commands: Commands,
    time: Res<Time>,
    despawn_at_q: Query<(Entity, &DespawnAt), With<DespawnAt>>,
) {
    for (entity, despawn_at) in despawn_at_q.iter() {
        if despawn_at.0 >= time.elapsed_secs() {
            continue;
        }
        commands.entity(entity).try_despawn();
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, despawn_expired);
}
