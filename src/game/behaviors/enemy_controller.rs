use crate::game::behaviors::dead::{Dead, DeadFor, DeadQueryData};
use crate::game::behaviors::despawn::Despawn;
use crate::game::behaviors::dynamic_character_controller::DynamicCharacterController;
use crate::game::behaviors::knocked_over::{
    KnockedOver, KnockedOverQueryData, KnockedOverSystemParams,
};
use crate::game::behaviors::pin_joint::{DisablePinJoints, EnablePinJoints};
use crate::game::behaviors::restore_data::RestorableQueryData;
use crate::game::behaviors::spawn_group::{
    SpawnGroup, SpawnGroupItem, SpawnGroupItemQueryData, SpawnGroupQueryData,
};
use crate::game::behaviors::stun::{OnStunned, OnUnStunned, StunSystemParam, StunTime, Stunned};
use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::effects::break_down_gltf::{BreakGltfParams, BreakGltfSystemParam};
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::enemy::{Enemy, EnemyAssets, PlayBoneSnap};
use crate::game::prefabs::game_world::GameWorld;
use crate::game::prefabs::game_world_markers::{GameWorldMarkerSystemParam, TempleBase};
use crate::game::rng::global::GlobalRng;
use crate::game::scenes::LevelData;
use crate::game::screens::Screen;
use avian3d::prelude::{
    AngularDamping, AngularVelocity, ColliderConstructor, CollisionStarted, Collisions,
    LinearDamping, LinearVelocity, LockedAxes, Restitution, RigidBody, SphericalJoint,
};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use std::time::Duration;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct EnemyController;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct Bone(Entity);

#[derive(QueryData)]
struct EnemyQueryData {
    entity: Entity,
    enemy: &'static Enemy,
    spawn_group_item: &'static SpawnGroupItem,
    target_ent: RestorableQueryData<TargetEnt>,
    locked_axes: RestorableQueryData<LockedAxes>,
}

impl EnemyQueryDataItem<'_> {
    pub fn store_and_remove(&self, entity_cmds: &mut EntityCommands) {
        self.target_ent.store_and_remove(entity_cmds);
        self.locked_axes.store_and_remove(entity_cmds);
    }
    pub fn restore(&self, entity_cmds: &mut EntityCommands) {
        self.target_ent.restore(entity_cmds);
        self.locked_axes.restore(entity_cmds);
    }
}

#[derive(QueryData)]
struct KnockedOverEnemyQueryData {
    enemy: EnemyQueryData,
    knocked_over: KnockedOverQueryData,
}

#[derive(QueryData)]
struct DeadEnemyQueryData {
    entity: Entity,
    enemy: EnemyQueryData,
    dead: DeadQueryData,
}

#[derive(SystemParam)]
struct EnemySystemParam<'w, 's> {
    enemies_q: Query<'w, 's, EnemyQueryData, With<EnemyController>>,
    velocity_q: Query<'w, 's, (&'static LinearVelocity, &'static AngularVelocity)>,
    break_gltf_sp: BreakGltfSystemParam<'w, 's>,
    stun_sp: StunSystemParam<'w, 's>,
    knocked_over_enemy_q: Query<
        'w,
        's,
        KnockedOverEnemyQueryData,
        (With<EnemyController>, Added<KnockedOver>, Without<Dead>),
    >,
    dead_q: Query<'w, 's, DeadEnemyQueryData, (With<EnemyController>, With<Dead>)>,
    knocked_over_sp: KnockedOverSystemParams<'w, 's>,
}

fn process_knocked_over(mut commands: Commands, mut level_data: ResMut<LevelData>,  enemy_sp: EnemySystemParam) {
    for item in enemy_sp.knocked_over_enemy_q.iter() {
        if !item
            .knocked_over
            .knocked_over
            .as_ref()
            .map(|ko| ko.is_added())
            .unwrap_or(false)
        {
            continue;
        }
        debug!(
            "knocked over: {} - angle {} >= {}",
            item.enemy.entity,
            item.knocked_over.current_pitch_angle().to_degrees(),
            item.knocked_over.knocked_over_angle.0.to_degrees()
        );

        level_data.kill_count += 1;

        commands
            .entity(item.enemy.entity)
            // prevents knocked over from updating
            .remove::<KnockedOver>()
            // prevents unstunning
            .remove::<Stunned>()
            .insert((
                Dead,
                Despawn::in_seconds(item.enemy.enemy.default_despawn_time()),
            ));
    }
}

fn process_dead(
    mut commands: Commands,
    mut enemy_sp: EnemySystemParam,
    game_world: Single<Entity, With<GameWorld>>,
) {
    for item in enemy_sp.dead_q.iter() {
        if !item.dead.dead.is_added() {
            continue;
        }
        let entity = item.entity;
        debug!("process_dead {entity}");
        let (lin_vel, ang_vel) = if let Ok((&lin_vel, &ang_vel)) = enemy_sp.velocity_q.get(entity) {
            (lin_vel, ang_vel)
        } else {
            (LinearVelocity::default(), AngularVelocity::default())
        };

        for bone in enemy_sp.break_gltf_sp.break_gltf(
            entity,
            BreakGltfParams {
                new_parent: Some(*game_world),
                ..Default::default()
            },
        ) {
            let despawn_in = item
                .dead
                .despawn
                .as_ref()
                .map(|item| item.ttl.as_secs_f32())
                .unwrap_or(item.enemy.enemy.default_despawn_time());
            commands.entity(bone).insert((
                Bone(entity),
                Dead,
                Despawn::in_seconds(despawn_in),
                RigidBody::Dynamic,
                ColliderConstructor::ConvexHullFromMesh,
                Restitution::new(0.001),
                LinearDamping(0.25),
                AngularDamping(0.25),
                LinearVelocity(lin_vel.0 / 100.0),
                AngularVelocity(ang_vel.0 / 100.0),
            ));
        }
    }
}

fn collision_force_check(
    mut commands: Commands,
    mut collision_started: EventReader<CollisionStarted>,
    mut rng: GlobalRng,
    enemy_assets: Res<EnemyAssets>,
    mut level_data: ResMut<LevelData>,
    collisions: Collisions,
    enemies: Query<Entity, With<Enemy>>,
    bowling_balls: Query<Entity, With<BowlingBall>>,
    temple_base_q: Single<Entity, With<TempleBase>>,
    children_q: Query<&Children>,
    mut enemy_sp: EnemySystemParam,
) {
    let temple_base = temple_base_q.into_inner();
    let mut temple_ents = vec![temple_base];
    for ent in children_q.iter_descendants(temple_base) {
        temple_ents.push(ent);
    }
    for &CollisionStarted(entity_a, entity_b) in collision_started.read() {
        let collided_entities = [entity_a, entity_b];

        let temple_collision_opt = collided_entities
            .iter()
            .find(|e| temple_ents.contains(e))
            .cloned();

        // Enemy reached the temple
        if temple_collision_opt.is_some() && collided_entities.iter().any(|&e| enemies.contains(e))
        {
            // Deal damage to temple
            level_data.temple_health = level_data.temple_health.saturating_sub(1);

            // Determine which one is the skele ent
            let skele = match temple_collision_opt.unwrap() == entity_a {
                false => entity_a,
                true => entity_b,
            };

            // Remove movement and add despawn timer
            commands
                .entity(skele)
                .remove::<TargetEnt>()
                .trigger(PlayBoneSnap)
                .trigger(DisablePinJoints)
                .insert(Despawn {
                    ttl: Duration::from_secs_f32(0.2),
                });
        }

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
        if collided_entities.iter().all(|&e| enemies.contains(e)) {
            // skip skele <-> skele
            // TODO: check kinetic force or create a joint based formation
            continue;
        }
        for skele in [entity_a, entity_b]
            .into_iter()
            .filter_map(|e| enemies.get(e).ok())
        {
            // TODO: only remove if enough force
            // TODO: if don't calc force for skele <-> skele
            //  we should make it so skele's maintain formation instead of converging and bumping into each other
            commands.entity(skele).trigger(PlayBoneSnap);
            enemy_sp.stun_sp.stun(skele);
        }
    }
}

fn on_stunned(
    trigger: Trigger<OnStunned>,
    mut commands: Commands,
    enemy: Query<EnemyQueryData, With<EnemyController>>,
) {
    let entity = trigger.target();
    debug!("Trigger<OnStunned> {entity}");
    let item = enemy
        .get(entity)
        .expect("Trigger<OnStunned> failed to resolve item - impossible");
    item.store_and_remove(&mut commands.entity(entity));
    commands.entity(entity).trigger(DisablePinJoints);
}

fn on_unstunned(
    trigger: Trigger<OnUnStunned>,
    mut commands: Commands,
    enemy: Query<EnemyQueryData, With<EnemyController>>,
) {
    let entity = trigger.target();
    debug!("Trigger<OnUnStunned> {entity}");
    let item = enemy
        .get(entity)
        .expect("Trigger<OnUnStunned> failed to resolve item - impossible");
    item.restore(&mut commands.entity(entity));
    commands.entity(entity).trigger(EnablePinJoints);
}

fn on_add_enemy_controller(trigger: Trigger<OnAdd, EnemyController>, mut commands: Commands) {
    debug!("Trigger<OnAdd, EnemyController> {}", trigger.target());
    commands
        .entity(trigger.target())
        .observe(on_stunned)
        .observe(on_unstunned);
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_add_enemy_controller);
    app.add_systems(
        PreUpdate,
        (process_knocked_over, process_dead, collision_force_check)
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}
