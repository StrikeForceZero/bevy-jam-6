use avian3d::prelude::{CollisionEventsEnabled, Gravity, LockedAxes};
use std::f32::consts::PI;

use crate::game::asset_tracking::LoadResource;
use crate::game::behaviors::dynamic_character_controller::{
    ControllerGravity, DynamicCharacterController, MaxSlopeAngle, MovementAcceleration,
    MovementDampingFactor,
};
use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::behaviors::{MaxMovementSpeed, MovementSpeed};
use crate::game::prefabs::bowling_ball::BowlingBall;
use avian3d::prelude::{CenterOfMass, Collider, CollisionStarted, Collisions, RigidBody};
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Resource, Asset, Debug, Clone, Reflect)]
pub struct EnemyAssets {
    #[dependency]
    pub base_skele: Handle<Gltf>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            base_skele: assets.load("models/enemies/LowPolySkeletonRigged.glb"),
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

impl Enemy {
    pub fn default_move_speed(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_MOVE_SPEED,
        }
    }
}

impl Enemy {
    pub fn default_max_move_speed(&self) -> f32 {
        match self {
            Self::BaseSkele => DEFAULT_MOVE_SPEED,
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();
    app.add_observer(on_enemy_added);
    app.add_systems(Update, collision_force_check);
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
        movement_speed,
        max_movement_speed,
        DynamicCharacterController,
        ControllerGravity::from(gravity),
        MaxSlopeAngle(60_f32.to_radians()),
    ));
}

fn collision_force_check(
    mut commands: Commands,
    mut collision_started: EventReader<CollisionStarted>,
    collisions: Collisions,
    enemies: Query<Entity, With<Enemy>>,
    bowling_balls: Query<Entity, With<BowlingBall>>,
) {
    for &CollisionStarted(entity_a, entity_b) in collision_started.read() {
        let collided_entities = [entity_a, entity_b];
        if !collided_entities
            .iter()
            .all(|&e| enemies.contains(e) || bowling_balls.contains(e))
        {
            // not skele <-> skele
            // not ball <-> skele
            continue;
        }
        if collided_entities.iter().all(|&e| bowling_balls.contains(e)) {
            // skip ball <-> ball
            continue;
        }
        for skele in [entity_a, entity_b]
            .into_iter()
            .filter_map(|e| enemies.get(e).ok())
        {
            // TODO: only remove if enough force
            // TODO: if don't calc force for skele <-> skele
            //  we should make it so skele's maintain formation instead of converging and bumping into each other
            commands
                .entity(skele)
                .remove::<TargetEnt>()
                .remove::<LockedAxes>();
        }
    }
}
