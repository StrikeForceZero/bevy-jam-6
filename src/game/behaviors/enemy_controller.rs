use crate::game::behaviors::dead::{Dead, DeadAndDespawnIn, DeadAt, DeadQueryData};
use crate::game::behaviors::knocked_over::{
    KnockedOver, KnockedOverQueryData, KnockedOverSystemParams,
};
use crate::game::behaviors::restore_data::RestorableQueryData;
use crate::game::behaviors::stun::{OnStunned, OnUnStunned, StunSystemParam};
use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::effects::break_down_gltf::BreakGltfSystemParam;
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::enemy::Enemy;
use avian3d::prelude::{
    AngularDamping, AngularVelocity, ColliderConstructor, CollisionStarted, Collisions,
    LinearDamping, LinearVelocity, LockedAxes, Restitution, RigidBody,
};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

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

fn process_knocked_over(mut commands: Commands, enemy_sp: EnemySystemParam) {
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
        commands
            .entity(item.enemy.entity)
            // prevents knocked over from updating
            .remove::<KnockedOver>()
            .insert(DeadAndDespawnIn(item.enemy.enemy.default_despawn_time()));
    }
}

fn process_dead(mut commands: Commands, mut enemy_sp: EnemySystemParam) {
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

        for bone in enemy_sp.break_gltf_sp.break_gltf(entity, true) {
            let despawn_in = item
                .dead
                .dead_and_despawn_in_opt
                .as_ref()
                .map(|item| item.0)
                .unwrap_or(item.enemy.enemy.default_despawn_time());
            commands.entity(bone).insert((
                Bone(entity),
                DeadAndDespawnIn(despawn_in),
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
    collisions: Collisions,
    enemies: Query<Entity, With<EnemyController>>,
    bowling_balls: Query<Entity, With<BowlingBall>>,
    mut enemy_sp: EnemySystemParam,
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
        (process_knocked_over, process_dead, collision_force_check).chain(),
    );
}
