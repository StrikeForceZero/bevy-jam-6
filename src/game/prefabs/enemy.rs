use crate::game::asset_tracking::LoadResource;
use crate::game::behaviors::dead::DeadAt;
use crate::game::behaviors::dynamic_character_controller::{
    ControllerGravity, DynamicCharacterController, MaxSlopeAngle,
};
use crate::game::behaviors::enemy_controller::EnemyController;
use crate::game::behaviors::grounded::Groundable;
use crate::game::behaviors::knocked_over::KnockedOverAngle;
use crate::game::behaviors::stun::{OnUnStunned, StunSystemParam};
use crate::game::behaviors::zap_stuns::ZapStunTime;
use crate::game::behaviors::{MaxMovementSpeed, MovementSpeed};
use crate::game::effects::break_down_gltf::BreakableGltfs;
use avian3d::prelude::{
    AngularVelocity, CenterOfMass, Collider, ColliderConstructor, CollisionStarted, Collisions,
    LinearDamping, LinearVelocity, LockedAxes, Restitution, RigidBody,
};
use avian3d::prelude::{CollisionEventsEnabled, Gravity};
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use std::f32::consts::PI;
use std::fmt::Debug;

#[auto_register_type]
#[derive(Resource, Asset, Debug, Clone, Reflect)]
pub struct EnemyAssets {
    #[dependency]
    pub base_skele: Handle<Gltf>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let gltf_handle = assets.load("models/enemies/LowPolySkeletonRigged.glb");
        world
            .resource_mut::<BreakableGltfs>()
            .add(gltf_handle.clone());
        Self {
            base_skele: gltf_handle,
        }
    }
}

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[require(CollisionEventsEnabled)]
pub enum Enemy {
    BaseSkele,
}

const DEFAULT_MOVE_SPEED: f32 = 30.0;
const DEFAULT_STUN_TIME: f32 = 2.0;
const DEFAULT_DESPAWN_AFTER_DEAD_SECS: f32 = 5.0;

impl Enemy {
    pub fn default_move_speed(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_MOVE_SPEED,
        }
    }
    pub fn default_max_move_speed(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_MOVE_SPEED,
        }
    }
    pub fn default_stun_time(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_STUN_TIME,
        }
    }
    pub fn default_despawn_time(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_DESPAWN_AFTER_DEAD_SECS,
        }
    }
}

fn on_enemy_added(
    trigger: Trigger<OnAdd, Enemy>,
    query: Query<&Enemy>,
    enemy_assets: Res<EnemyAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut commands: Commands,
    gravity: Res<Gravity>,
) {
    let enemy = query
        .get(trigger.target())
        .expect("No target entity for trigger");

    // Model handle
    let gltf_h = match *enemy {
        Enemy::BaseSkele => enemy_assets.base_skele.clone(),
    };
    let gltf = gltfs
        .get(&gltf_h)
        .unwrap_or_else(|| panic!("Missing gltf asset for {:?}", enemy));

    // MovementSpeed
    let movement_speed = MovementSpeed(enemy.default_move_speed());
    let max_movement_speed = MaxMovementSpeed(enemy.default_max_move_speed());

    commands.entity(trigger.target()).insert((
        children![(
            SceneRoot(gltf.scenes[0].clone()),
            // For some reason the skele meshes are 180 rotated so fixing it
            // with a local transform.
            Transform::from_rotation(Quat::from_rotation_y(PI)).with_translation(Vec3::Y * -1.75),
        ),],
        // Parry colliders are centered around origin. Meshes have lowest
        // vertex at y=0.0. Spawning the collider allows us to adjust
        // its position to match the mesh.
        Collider::capsule(0.25, 3.0),
        CenterOfMass::new(0.0, -5.5, 0.0),
        RigidBody::Dynamic,
        EnemyController,
        movement_speed,
        max_movement_speed,
        DynamicCharacterController,
        Groundable,
        KnockedOverAngle(75_f32.to_radians()),
        ZapStunTime(enemy.default_stun_time()),
        ControllerGravity::from(gravity),
        MaxSlopeAngle(60_f32.to_radians()),
    ));
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();
    app.add_observer(on_enemy_added);
}
