use super::MovementSpeed;
use crate::game::pause_controller::PausableSystems;
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::enemy::Enemy;
use avian3d::prelude::CollidingEntities;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(CollidingEntities)]
pub struct TargetEnt {
    pub target_ent: Entity,
    pub within_distance: f32,
}

fn target_ent_sys(
    mut commands: Commands,
    time: Res<Time>,
    target_q: Query<(
        Entity,
        &TargetEnt,
        &CollidingEntities,
        Option<&MovementSpeed>,
    )>,
    bowling_ball: Query<&BowlingBall>,
    enemies: Query<&Enemy>,
    mut transform_q: Query<&mut Transform>,
) {
    for (self_ent, &target, colliding_entities, movement_speed) in target_q.iter() {
        // don't try to move if hit by bowling ball
        if colliding_entities
            .0
            .iter()
            .any(|&c| bowling_ball.contains(c) || enemies.contains(c))
        {
            // TODO: we should only temporarily block moving and see if the enemy is knocked down
            commands.entity(self_ent).remove::<TargetEnt>();
            continue;
        }
        let target_ent = target.target_ent;
        // If target ent no longer exists, remove component
        let Ok(target_trans) = transform_q.get(target_ent).cloned() else {
            commands.entity(self_ent).remove::<TargetEnt>();
            return;
        };
        let transform = transform_q
            .get(self_ent)
            .expect("no transform found for TargetEnt");
        // Remove y component as some objects are not at ground level (e.g.
        // tower center is at this point in time in the middle of its mesh).
        let target_trans = target_trans.with_translation(Vec3::new(
            target_trans.translation.x,
            transform.translation.y,
            target_trans.translation.z,
        ));

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
    app.add_systems(Update, target_ent_sys.in_set(PausableSystems));
}
