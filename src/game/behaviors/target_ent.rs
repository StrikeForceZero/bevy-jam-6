use super::MovementSpeed;
use crate::game::behaviors::dynamic_character_controller::{
    DynamicCharacterController, MovementAction, MovementActionEvent,
};
use crate::game::pause_controller::PausableSystems;
use crate::game::utils::vector::XYZ_3D;
use avian3d::prelude::{CollidingEntities, LinearVelocity};
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
    time: Res<Time<Fixed>>,
    self_q: Query<(
        Entity,
        &TargetEnt,
        Option<&MovementSpeed>,
        Option<&LinearVelocity>,
        Has<DynamicCharacterController>,
    )>,
    mut transform_q: Query<(Mut<Transform>, &GlobalTransform)>,
    mut movement_action: EventWriter<MovementActionEvent>,
) {
    let delta_time: f32 = time.delta_secs();

    for (self_ent, target, opt_speed, linear_velocity_opt, has_controller) in self_q.iter() {
        let target_ent = target.target_ent;
        let (_, target_gtx) = match transform_q.get(target_ent) {
            Ok((tx, gtx)) => (*tx, *gtx),
            Err(err) => {
                error!(
                    "TargetEnt {self_ent} target {target_ent} does not have Transform - {err:?}"
                );
                commands.entity(self_ent).remove::<TargetEnt>();
                continue;
            }
        };

        let (mut self_tx, &self_gtx) = match transform_q.get_mut(self_ent) {
            Ok(tx) => tx,
            Err(err) => {
                error!("TargetEnt {self_ent} without Transform - {err:?}");
                commands.entity(self_ent).remove::<TargetEnt>();
                continue;
            }
        };

        let target_pos_global = {
            let mut flat_target = target_gtx.translation();
            // limit to horizontal only movements
            flat_target.y = self_gtx.translation().y;
            flat_target
        };

        let target_pos_local = target_pos_global - self_gtx.translation();

        // Face the target
        if target_pos_local.length_squared() > 0.0 {
            let yaw = f32::atan2(target_pos_local.x, target_pos_local.z);
            let current_tx = *self_tx;
            let mut looking_at_transform = current_tx;
            looking_at_transform.rotation = Quat::from_rotation_y(-yaw);
            if looking_at_transform != current_tx {
                *self_tx = looking_at_transform;
            }
        }

        let to_target: Vec3 = target_pos_global - self_gtx.translation();
        let dist: f32 = to_target.length();

        let is_moving = dist > target.within_distance;
        let is_in_attack_range = !is_moving;

        if is_moving {
            if let Some(&MovementSpeed(full_speed)) = opt_speed {
                let max_dist_cover: f32 = full_speed * delta_time; // meters this tick
                let dist_to_cover: f32 = dist - target.within_distance;
                let dir: Vec3 = to_target / dist;

                if has_controller {
                    let clamped_speed: f32 = (dist_to_cover / delta_time).min(full_speed);
                    let clamped_dir = dir * clamped_speed;
                    let movement_amount = XYZ_3D::from(clamped_dir).xz();
                    movement_action.write(MovementActionEvent::new(
                        self_ent,
                        MovementAction::Walk(movement_amount),
                    ));
                } else {
                    // clamped so we don’t overshoot:
                    let actual_move: f32 = dist_to_cover.min(max_dist_cover);
                    self_tx.translation += dir * actual_move;
                }
                continue;
            };
        }
        if has_controller {
            let lin_vel = linear_velocity_opt.copied().unwrap_or_default();
            if lin_vel.0.xz() != Vec2::ZERO {
                movement_action.write(MovementActionEvent::new(self_ent, MovementAction::Stop));
            }
        }
        if is_in_attack_range {
            // TODO: attack
            info_once!("TargetEnt {self_ent} attacking {target_ent}");
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, target_ent_sys.in_set(PausableSystems));
}
