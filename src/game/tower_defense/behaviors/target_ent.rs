use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use super::MovementSpeed;

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetEnt {
    pub target_ent: Entity,
    pub within_distance: f32,
}

fn target_ent_sys(
    mut commands: Commands,
    time: Res<Time>,
    target_q: Query<(Entity, &TargetEnt, Option<&MovementSpeed>)>,
    mut transform_q: Query<&mut Transform>,
) {
    for (self_ent, &target, movement_speed) in target_q.iter() {
        let target_ent = target.target_ent;
        // If target ent no longer exists, remove component
        let Ok(target_trans) = transform_q.get(target_ent).cloned() else {
            commands.entity(self_ent).remove::<TargetEnt>();
            return;
        };

        // Face target
        let mut self_trans = transform_q.get_mut(self_ent).unwrap();
        self_trans.look_at(target_trans.translation, Vec3::Y);

        // If target is outside range (`within_distance`), move towards it,
        // otherwise attack.
        let dist = self_trans.translation.distance(target_trans.translation);
        if dist > target.within_distance {
            if let Some(move_speed) = movement_speed {
                let move_speed = move_speed.0 * time.delta_secs();
                let move_dist = move_speed.min(dist - target.within_distance);
                self_trans.translation = self_trans
                    .translation
                    .move_towards(target_trans.translation, move_dist);
            }
        } else {
            // TODO trigger attack
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, target_ent_sys);
}
