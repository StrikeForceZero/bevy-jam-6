use crate::game::asset_tracking::LoadResource;
use crate::game::behaviors::dynamic_character_controller::{
    ControllerGravity, DynamicCharacterController, Grounded, MaxSlopeAngle, MovementAcceleration,
    MovementDampingFactor,
};
use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::behaviors::{MaxMovementSpeed, MovementSpeed};
use crate::game::effects::lightning_ball::LightningBallZappedBy;
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::snapshot::Snapshot;
use crate::game::utils::quat::get_pitch_and_roll;
use avian3d::prelude::{CenterOfMass, Collider, CollisionStarted, Collisions, RigidBody};
use avian3d::prelude::{CollisionEventsEnabled, Gravity, LockedAxes};
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
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
const DEFAULT_STUN_TIME: f32 = 2.0;
const DESPAWN_AFTER_DEAD_SECS: f32 = 5.0;

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
}

// TODO: Move?
#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct DeadAt(pub f32);

// TODO: Move / Cleanup?
#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[component(on_remove=Self::on_remove)]
pub struct Stunned;

impl Stunned {
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .try_remove::<StunnedAt>()
            .try_remove::<StunnedData<TargetEnt>>()
            .try_remove::<StunnedData<LockedAxes>>();
    }
}

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[require(Stunned)]
pub struct StunnedAt(pub f32);

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
pub struct StunTime(pub f32);

#[auto_register_type(StunnedData<TargetEnt>)]
#[auto_register_type(StunnedData<LockedAxes>)]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[require(Stunned)]
pub struct StunnedData<T>(T)
where
    T: Component + Debug + Copy + Clone + Reflect;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();
    app.add_observer(on_enemy_added);
    app.add_systems(
        Update,
        (
            clear_dead,
            refresh_zapped_by,
            knocked_over,
            unstun,
            collision_force_check,
        ),
    );
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
    let stun_time = StunTime(enemy.default_stun_time());

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
        stun_time,
        DynamicCharacterController,
        ControllerGravity::from(gravity),
        MaxSlopeAngle(60_f32.to_radians()),
    ));
}

#[derive(QueryData)]
struct EnemyStunnedAtQueryData {
    entity: Entity,
    stunned_at: &'static StunnedAt,
    stunned_data_target_ent_opt: Option<&'static StunnedData<TargetEnt>>,
    stunned_data_locked_axis_opt: Option<&'static StunnedData<LockedAxes>>,
    transform: Ref<'static, Transform>,
    is_grounded: Has<Grounded>,
}

#[derive(QueryData)]
struct EnemyBeingStunnedQueryData {
    stun_time: &'static StunTime,
    target_ent: Option<&'static TargetEnt>,
    locked_axes: Option<&'static LockedAxes>,
}

#[derive(SystemParam)]
struct StunSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    stunned_q: Query<'w, 's, EnemyStunnedAtQueryData, With<Enemy>>,
    data_q: Query<'w, 's, EnemyBeingStunnedQueryData, With<Enemy>>,
    zapped_by: Query<'w, 's, (Entity, Ref<'static, LightningBallZappedBy>)>,
    dead_q: Query<'w, 's, (Entity, &'static DeadAt), With<Enemy>>,
}

impl StunSystemParam<'_, '_> {
    fn refresh_zapped_by(&mut self) {
        let mut stunned = EntityHashSet::default();
        for (entity, zapped_by) in self.zapped_by.iter() {
            if zapped_by.is_added() {
                stunned.insert(entity);
            }
        }
        for entity in stunned.into_iter() {
            self.stun(entity);
        }
    }
    fn unstun_expired(&mut self) {
        for stunned in self.stunned_q.iter() {
            if !stunned.is_grounded || stunned.stunned_at.0 > self.time.elapsed_secs_wrapped() {
                continue;
            }
            debug!("unstunning entity: {}", stunned.entity);
            let mut entity_cmds = self.commands.entity(stunned.entity);
            entity_cmds.remove::<Stunned>();
            if let Some(data) = stunned.stunned_data_target_ent_opt {
                entity_cmds.insert(data.0);
            }
            if let Some(data) = stunned.stunned_data_locked_axis_opt {
                entity_cmds.insert(data.0);
            }
        }
    }
    fn stun(&mut self, entity: Entity) {
        if let Ok(data) = self.data_q.get(entity) {
            self.stun_with_time(entity, data.stun_time.0);
        } else {
            error!("missing StunTime for {entity}");
        }
    }
    fn stun_with_time(&mut self, entity: Entity, stun_duration: f32) {
        let Ok(item) = self.data_q.get(entity) else {
            error!("attempted to stun entity that doesn't exist or is not Enemy: {entity}");
            return;
        };
        let mut entity_cmds = self.commands.entity(entity);
        debug!("stunning entity: {entity}");
        entity_cmds
            .insert(Stunned)
            .insert(StunnedAt(self.time.elapsed_secs() + stun_duration));
        if let Some(&target_ent) = item.target_ent {
            entity_cmds
                .insert(StunnedData(target_ent))
                .remove::<TargetEnt>();
        }
        if let Some(&locked_axes) = item.locked_axes {
            entity_cmds
                .insert(StunnedData(locked_axes))
                .remove::<LockedAxes>();
        }
    }
    fn handle_knocked_over(&mut self) {
        let mut dead_entities = EntityHashSet::default();
        for stunned in self.stunned_q.iter() {
            let rot = stunned.transform.rotation;
            let (pitch, roll) = get_pitch_and_roll(rot);
            let pitch_angle = pitch.to_degrees().abs();
            if pitch_angle >= 60_f32 {
                dead_entities.insert(stunned.entity);
            }
        }
        for entity in dead_entities.into_iter() {
            debug!("knocked over entity: {}", entity);
            self.commands
                .entity(entity)
                .remove::<Stunned>()
                .insert(DeadAt(self.time.elapsed_secs()));
        }
    }
    fn clear_dead(&mut self) {
        let mut dead_entities = EntityHashSet::default();
        for (entity, dead_at) in self.dead_q.iter() {
            if dead_at.0 + DESPAWN_AFTER_DEAD_SECS > self.time.elapsed_secs_wrapped() {
                continue;
            }
            dead_entities.insert(entity);
        }
        for entity in dead_entities.into_iter() {
            self.commands.entity(entity).despawn();
        }
    }
}

fn unstun(mut stun_system_param: StunSystemParam) {
    stun_system_param.unstun_expired();
}

fn refresh_zapped_by(mut stun_system_param: StunSystemParam) {
    stun_system_param.refresh_zapped_by();
}

fn knocked_over(mut stun_system_param: StunSystemParam) {
    stun_system_param.handle_knocked_over();
}

fn clear_dead(mut stun_system_param: StunSystemParam) {
    stun_system_param.clear_dead();
}

fn collision_force_check(
    mut commands: Commands,
    mut collision_started: EventReader<CollisionStarted>,
    collisions: Collisions,
    enemies: Query<Entity, With<Enemy>>,
    bowling_balls: Query<Entity, With<BowlingBall>>,
    mut stun_sp: StunSystemParam,
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
            stun_sp.stun(skele);
        }
    }
}
