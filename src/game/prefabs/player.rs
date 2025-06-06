use crate::game::asset_tracking::LoadResource;
use crate::game::camera::CameraTarget;
use crate::game::effects::lightning_ball::LightningBall;
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::game_world::GameWorld;
use crate::game::prefabs::game_world_markers::{
    BowlingBallSpawnMarker, ComponentName, EntityWithGlobalTransformQueryData, SpawnHelper,
};
use avian3d::prelude::{Collider, ExternalAngularImpulse, ExternalImpulse, Mass, RigidBody};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[require(Visibility)]
#[require(RigidBody::Kinematic)]
pub struct Player;

impl ComponentName for Player {
    fn component_name() -> &'static str {
        "Player"
    }
}

#[auto_register_type]
#[derive(Resource, Asset, Debug, Clone, Reflect)]
pub struct PlayerAssets {
    #[dependency]
    pub scene: Handle<Scene>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            scene: assets.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/zeus/zeus_rigged_manual_bowling_ball.glb"),
            ),
        }
    }
}

#[derive(SystemParam)]
pub struct PlayerSystemParam<'w, 's> {
    pub player_transform: Single<'w, Ref<'static, Transform>, With<Player>>,
    player: SpawnHelper<'w, 's, GameWorld, Player>,
    pub bowling_ball_spawn: SpawnHelper<'w, 's, GameWorld, BowlingBallSpawnMarker>,
}

impl PlayerSystemParam<'_, '_> {
    pub fn entity(&self) -> Entity {
        self.player.target_q.entity
    }
    pub fn get_player_rotation(&self) -> Quat {
        self.player.compute_target_local_transform(None).rotation
    }
    pub fn spawn_bowling_ball_spawn(
        &mut self,
        bundle: impl Bundle,
        transform: Option<Transform>,
    ) -> Entity {
        self.bowling_ball_spawn.spawn_in(bundle, transform)
    }
    pub fn spawn_bowling_ball(&mut self, power: f32, accuracy_offset_radians: f32) -> Entity {
        let player_rot = self.get_player_rotation();
        let accuracy_rot = player_rot * Quat::from_rotation_y(accuracy_offset_radians);
        let bowling_ball = self.spawn_bowling_ball_spawn(
            (
                BowlingBall,
                LightningBall,
                ExternalAngularImpulse::new(accuracy_rot * (Vec3::X * 10.0 * power)),
                ExternalImpulse::new(accuracy_rot * (Vec3::Z * 1000.0 * power)),
                Mass(20.0),
            ),
            Some(Transform::from_scale(Vec3::splat(20.0))),
        );
        bowling_ball
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();
    app.add_observer(on_added);
}

fn on_added(trigger: Trigger<OnAdd, Player>, assets: Res<PlayerAssets>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert((SceneRoot(assets.scene.clone()), Collider::capsule(3.0, 8.0)));
}
