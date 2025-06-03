use crate::game::{
    asset_tracking::LoadResource,
    health::{Health, MaxHealth},
    prefabs::health_bar::overhead_health_bar,
    spark::SparkTarget,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::behaviors::MovementSpeed;

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
            base_skele: assets.load("models/enemies/Skeleton_Minion.glb"),
        }
    }
}

impl EnemyAssets {
    pub fn get_by_enemy(&self, enemy: &Enemy) -> Handle<Gltf> {
        match enemy {
            Enemy::BaseSkele => self.base_skele.clone(),
        }
    }
}

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform, SparkTarget)]
pub enum Enemy {
    BaseSkele,
}

pub struct EnemyMeta {
    pub max_health: f32,
    pub move_speed: f32,
}

impl EnemyMeta {
    fn new(max_health: f32, move_speed: f32) -> Self {
        Self {
            max_health,
            move_speed,
        }
    }
}

impl Enemy {
    pub fn meta(&self) -> EnemyMeta {
        match self {
            Self::BaseSkele => EnemyMeta::new(100.0, 16.0),
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<EnemyAssets>();
    app.add_observer(Enemy::handle_on_add);
}

impl Enemy {
    fn handle_on_add(
        trigger: Trigger<OnAdd, Self>,
        query: Query<&Self, Added<Self>>,
        enemy_assets: Res<EnemyAssets>,
        gltfs: Res<Assets<Gltf>>,
        mut commands: Commands,
    ) {
        let enemy = query.get(trigger.target()).expect("OnAdd broken");

        let Some(gltf) = gltfs.get(&enemy_assets.get_by_enemy(enemy)) else {
            panic!("Missing gltf asset for {:?}", enemy)
        };

        let enemy_meta = enemy.meta();

        commands.entity(trigger.target()).insert((
            SceneRoot(gltf.scenes[0].clone()),
            MovementSpeed(enemy_meta.move_speed),
            Health(enemy_meta.max_health),
            MaxHealth(enemy_meta.max_health),
            RigidBody::Dynamic,
            Collider::compound(vec![(
                Vec3::Y,
                Quat::default(),
                Collider::capsule(0.5, 1.6),
            )]),
            LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z()
                .lock_translation_y(),
            children![overhead_health_bar(trigger.target(), 20.0)],
        ));
    }
}
