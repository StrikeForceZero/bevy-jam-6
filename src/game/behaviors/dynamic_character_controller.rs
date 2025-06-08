// derived from https://github.com/Jondolf/avian/blob/e6c0d4dad1f6b8a50d65c7e3d15ec5a4282d5045/crates/avian3d/examples/kinematic_character_3d/plugin.rs#L7

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::behaviors::grounded::Grounded;
use crate::game::behaviors::{MaxMovementSpeed, clamp_velocity_to_max_xz};
use crate::game::utils::vector::XZ_3D;
use avian3d::{
    math::*,
    prelude::{NarrowPhaseSet, *},
};
use bevy::ecs::query::QueryData;
use smart_default::SmartDefault;

/// An event sent for a movement input action.
#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Copy, Clone, Reflect)]
pub struct MovementActionEvent {
    entity: Entity,
    action: MovementAction,
}

impl MovementActionEvent {
    pub fn new(entity: Entity, action: MovementAction) -> Self {
        Self { entity, action }
    }
}

#[auto_register_type]
#[derive(Debug, Copy, Clone, Reflect)]
pub enum MovementAction {
    Walk(XZ_3D),
    Jump,
    Stop,
}

/// A marker component indicating that an entity is using a character controller.
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(RigidBody::Dynamic)]
#[require(Collider)]
#[require(ShapeCaster)]
#[require(ControllerGravity)]
#[require(MovementAcceleration)]
#[require(MovementDampingFactor)]
#[require(JumpImpulse)]
#[require(MaxSlopeAngle)]
#[require(ControllerMode)]
#[require(LockedAxes::ROTATION_LOCKED)]
pub struct DynamicCharacterController;

/// The acceleration used for character movement.
#[auto_register_type]
#[derive(Component, Debug, SmartDefault, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct MovementAcceleration(#[default = 1.0] pub Scalar);

/// The damping factor used for slowing down movement.
#[auto_register_type]
#[derive(Component, Debug, SmartDefault, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct MovementDampingFactor(#[default = 1.0] pub Scalar);

/// The strength of a jump.
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct JumpImpulse(pub Scalar);

/// The gravitational acceleration used for a character controller.
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct ControllerGravity(pub Vector);

impl From<Res<'_, Gravity>> for ControllerGravity {
    fn from(gravity: Res<Gravity>) -> Self {
        Self(gravity.0)
    }
}

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub enum ControllerMode {
    #[default]
    Velocity,
    Force,
}

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct MaxSlopeAngle(pub Scalar);

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time<Fixed>>,
    mut movement_event_reader: EventReader<MovementActionEvent>,
    mut controllers: Query<(
        Entity,
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &ControllerMode,
        Option<&MaxMovementSpeed>,
        Has<Grounded>,
    )>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for event in movement_event_reader.read() {
        let Ok((
            entity,
            movement_acceleration,
            jump_impulse,
            mut linear_velocity,
            mut external_force,
            mut external_impulse,
            controller_mode,
            max_movement_speed_opt,
            is_grounded,
        )) = controllers.get_mut(event.entity)
        else {
            error!(
                "MovementActionEvent sent for {} without controller",
                event.entity
            );
            continue;
        };
        let original_linear_velocity = linear_velocity.0;
        match event.action {
            MovementAction::Walk(direction) => {
                if !is_grounded {
                    continue;
                }
                let x = direction.x() * movement_acceleration.0 * delta_time;
                let z = direction.z() * movement_acceleration.0 * delta_time;
                if x != 0.0 {
                    match controller_mode {
                        ControllerMode::Velocity => {
                            linear_velocity.x += x;
                        }
                        ControllerMode::Force => {
                            external_force.x += x;
                        }
                    }
                }
                if z != 0.0 {
                    match controller_mode {
                        ControllerMode::Velocity => {
                            linear_velocity.z += z;
                        }
                        ControllerMode::Force => {
                            external_force.z += z;
                        }
                    }
                }
                if let Some(max_movement_speed) = max_movement_speed_opt {
                    let clamped_velocity =
                        clamp_velocity_to_max_xz(*linear_velocity, max_movement_speed.0);
                    if clamped_velocity.0 != linear_velocity.0 {
                        trace!(
                            "entity {entity} clamped velocity: {:?} -> {clamped_velocity:?}",
                            linear_velocity.0
                        );
                        linear_velocity.0 = clamped_velocity.0;
                    }
                }
            }
            MovementAction::Jump => {
                if is_grounded && linear_velocity.y != jump_impulse.0 {
                    match controller_mode {
                        ControllerMode::Velocity => {
                            linear_velocity.y = jump_impulse.0;
                        }
                        ControllerMode::Force => {
                            external_impulse.y = jump_impulse.0;
                        }
                    }
                }
            }
            MovementAction::Stop => {
                if linear_velocity.0.xz() != Vec2::ZERO {
                    linear_velocity.x = 0.0;
                    linear_velocity.z = 0.0;
                }
            }
        }
        if original_linear_velocity != linear_velocity.0 {
            trace!(
                "entity {entity} movement applied: {original_linear_velocity:?} -> {linear_velocity:?} ({:?})",
                event.action
            );
        }
    }
}

/// Applies [`ControllerGravity`] to character controllers.
fn apply_gravity(
    time: Res<Time<Fixed>>,
    mut controllers: Query<(&ControllerGravity, &mut LinearVelocity)>,
) {
    // Precision is adjusted so that the example works with
    // both the `f32` and `f64` features. Otherwise you don't need this.
    let delta_time = time.delta_secs_f64().adjust_precision();

    for (gravity, mut linear_velocity) in &mut controllers {
        linear_velocity.0 += gravity.0 * delta_time;
    }
}

/// Slows down movement in the XZ plane.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        if damping_factor.0 <= 0.0 {
            continue;
        }
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
    }
}

/// Kinematic bodies do not get pushed by collisions by default,
/// so it needs to be done manually.
///
/// This system handles collision response for kinematic character controllers
/// by pushing them along their contact normals by the current penetration depth,
/// and applying velocity corrections in order to snap to slopes, slide along walls,
/// and predict collisions using speculative contacts.
#[allow(clippy::type_complexity)]
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut character_controllers: Query<
        (&mut Position, &mut LinearVelocity, Option<&MaxSlopeAngle>),
        (With<RigidBody>, With<DynamicCharacterController>),
    >,
    time: Res<Time>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for contacts in collisions.iter() {
        // Get the rigid body entities of the colliders (colliders could be children)
        let Ok([&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }]) =
            collider_rbs.get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Get the body of the character controller and whether it is the first
        // or second entity in the collision.
        let is_first: bool;

        let character_rb: RigidBody;
        let is_other_dynamic: bool;

        let (mut position, mut linear_velocity, max_slope_angle) =
            if let Ok(character) = character_controllers.get_mut(rb1) {
                is_first = true;
                character_rb = *bodies.get(rb1).unwrap();
                is_other_dynamic = bodies.get(rb2).is_ok_and(|rb| rb.is_dynamic());
                character
            } else if let Ok(character) = character_controllers.get_mut(rb2) {
                is_first = false;
                character_rb = *bodies.get(rb2).unwrap();
                is_other_dynamic = bodies.get(rb1).is_ok_and(|rb| rb.is_dynamic());
                character
            } else {
                continue;
            };

        // This system only handles collision response for kinematic character controllers.
        if !character_rb.is_kinematic() {
            continue;
        }

        // Iterate through contact manifolds and their contacts.
        // Each contact in a single manifold shares the same contact normal.
        for manifold in contacts.manifolds.iter() {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            let mut deepest_penetration: Scalar = Scalar::MIN;

            // Solve each penetrating contact in the manifold.
            for contact in manifold.points.iter() {
                if contact.penetration > 0.0 {
                    position.0 += normal * contact.penetration;
                }
                deepest_penetration = deepest_penetration.max(contact.penetration);
            }

            // For now, this system only handles velocity corrections for collisions against static geometry.
            if is_other_dynamic {
                continue;
            }

            // Determine if the slope is climbable or if it's too steep to walk on.
            let slope_angle = normal.angle_between(Vector::Y);
            let climbable = max_slope_angle.is_some_and(|angle| slope_angle.abs() <= angle.0);

            if deepest_penetration > 0.0 {
                // If the slope is climbable, snap the velocity so that the character
                // up and down the surface smoothly.
                if climbable {
                    // Points in the normal's direction in the XZ plane.
                    let normal_direction_xz =
                        normal.reject_from_normalized(Vector::Y).normalize_or_zero();

                    // The movement speed along the direction above.
                    let linear_velocity_xz = linear_velocity.dot(normal_direction_xz);

                    // Snap the Y speed based on the speed at which the character is moving
                    // up or down the slope, and how steep the slope is.
                    //
                    // A 2D visualization of the slope, the contact normal, and the velocity components:
                    //
                    //             ╱
                    //     normal ╱
                    // *         ╱
                    // │   *    ╱   velocity_x
                    // │       * - - - - - -
                    // │           *       | velocity_y
                    // │               *   |
                    // *───────────────────*

                    let max_y_speed = -linear_velocity_xz * slope_angle.tan();
                    linear_velocity.y = linear_velocity.y.max(max_y_speed);
                } else {
                    // The character is intersecting an unclimbable object, like a wall.
                    // We want the character to slide along the surface, similarly to
                    // a collide-and-slide algorithm.

                    // Don't apply an impulse if the character is moving away from the surface.
                    if linear_velocity.dot(normal) > 0.0 {
                        continue;
                    }

                    // Slide along the surface, rejecting the velocity along the contact normal.
                    let impulse = linear_velocity.reject_from_normalized(normal);
                    linear_velocity.0 = impulse;
                }
            } else {
                // The character is not yet intersecting the other object,
                // but the narrow phase detected a speculative collision.
                //
                // We need to push back the part of the velocity
                // that would cause penetration within the next frame.

                let normal_speed = linear_velocity.dot(normal);

                // Don't apply an impulse if the character is moving away from the surface.
                if normal_speed > 0.0 {
                    continue;
                }

                // Compute the impulse to apply.
                let impulse_magnitude =
                    normal_speed - (deepest_penetration / time.delta_secs_f64().adjust_precision());
                let mut impulse = impulse_magnitude * normal;

                // Apply the impulse differently depending on the slope angle.
                if climbable {
                    // Avoid sliding down slopes.
                    linear_velocity.y -= impulse.y.min(0.0);
                } else {
                    // Avoid climbing up walls.
                    impulse.y = impulse.y.max(0.0);
                    linear_velocity.0 -= impulse;
                }
            }
        }
    }
}

#[derive(QueryData)]
pub struct DynamicCharacterControllerQueryData {
    pub entity: Entity,
    pub character_controller: &'static DynamicCharacterController,
    pub movement_acceleration: &'static MovementAcceleration,
    pub movement_damping_factor: &'static MovementDampingFactor,
    pub jump_impulse: &'static JumpImpulse,
    pub controller_gravity: &'static ControllerGravity,
    pub locked_axes: Option<&'static LockedAxes>,
    pub max_slope_angle: Option<&'static MaxSlopeAngle>,
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (apply_gravity, movement, apply_movement_damping).chain(),
    );
    app.add_systems(
        // Run collision handling after collision detection.
        //
        // NOTE: The collision implementation here is very basic and a bit buggy.
        //       A collide-and-slide algorithm would likely work better.
        PhysicsSchedule,
        kinematic_controller_collisions.in_set(NarrowPhaseSet::Last),
    );
}
