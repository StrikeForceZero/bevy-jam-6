use crate::game::behaviors::dynamic_character_controller::MaxSlopeAngle;
use avian3d::math::{Quaternion, Vector};
use avian3d::prelude::{Collider, Rotation, ShapeCaster, ShapeHits};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::transform::systems::propagate_parent_transforms;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(PreviousScale)]
pub struct Groundable;

/// A marker component indicating that an entity is on the ground.
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Groundable)]
#[component(storage = "SparseSet")]
pub struct Grounded;

/// used to determine when the scale has updated
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
struct PreviousScale(Option<Vec3>);

#[derive(QueryData)]
pub struct GroundedQueryData {
    pub is_groundable: Has<Groundable>,
    pub is_grounded: Has<Grounded>,
}

impl GroundedQueryDataItem<'_> {
    pub fn is_grounded(&self) -> Option<bool> {
        if self.is_groundable {
            Some(self.is_grounded)
        } else {
            None
        }
    }
}

#[derive(SystemParam)]
pub struct GroundedSystemParam<'w, 's> {
    grounded_q: Query<'w, 's, GroundedQueryData>,
}

impl GroundedSystemParam<'_, '_> {
    pub fn is_grounded(&self, entity: Entity) -> Option<bool> {
        self.grounded_q
            .get(entity)
            .ok()
            .as_ref()
            .and_then(GroundedQueryDataItem::is_grounded)
    }
}

pub fn ground_caster(collider: &Collider, scale: Vec3) -> ShapeCaster {
    // Create shape caster as a slightly smaller version of collider
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(scale * 0.99, 10);
    ShapeCaster::new(
        caster_shape,
        Vector::ZERO,
        Quaternion::default(),
        Dir3::NEG_Y,
    )
    .with_max_distance(scale.y * 0.2)
}

/// Updates ground caster when the collider is updated
fn update_ground_caster(
    trigger: Trigger<OnInsert, (Collider, Groundable)>,
    self_q: Query<(&Collider, &Transform), With<Groundable>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((collider, transform)) = self_q.get(entity) else {
        return;
    };
    debug!("creating ground caster for {entity}");
    commands
        .entity(entity)
        .insert(ground_caster(collider, transform.scale));
}

fn update_scale(
    mut commands: Commands,
    updated_transforms: Query<
        (Entity, Ref<Collider>, Ref<Transform>, Ref<PreviousScale>),
        With<Groundable>,
    >,
) {
    for (entity, collider, transform, prev_scale) in updated_transforms.iter() {
        let prev_scale_opt = prev_scale.0;
        let scale_changed = Some(transform.scale) != prev_scale_opt;
        if !scale_changed && !transform.is_changed() {
            continue;
        }
        let prev_scale = prev_scale_opt.unwrap_or(transform.scale);
        let scale_changed = scale_changed || transform.scale != prev_scale;
        if !scale_changed {
            continue;
        }
        debug!(
            "updating scale and ground_caster for {entity} from {prev_scale_opt:?} to {:?}",
            Some(prev_scale)
        );
        commands.entity(entity).insert((
            PreviousScale(Some(prev_scale)),
            ground_caster(&collider, transform.scale),
        ));
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ShapeHits,
            &Rotation,
            Option<&MaxSlopeAngle>,
            Has<Grounded>,
        ),
        With<Groundable>,
    >,
) {
    for (entity, hits, rotation, max_slope_angle, was_grounded) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                trace!(
                    "{entity} hit normal: {}, angle_between: {} <= {}",
                    hit.normal2,
                    (rotation * -hit.normal2).angle_between(Vector::Y).abs(),
                    angle.0
                );
                (rotation * -hit.normal2).angle_between(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded != was_grounded {
            trace!("entity {entity} grounded: {was_grounded} -> {is_grounded}");
            if is_grounded {
                commands.entity(entity).insert(Grounded);
            } else {
                commands.entity(entity).remove::<Grounded>();
            }
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, update_grounded);
    app.add_systems(PostUpdate, update_scale.after(propagate_parent_transforms));
    app.add_observer(update_ground_caster);
}
