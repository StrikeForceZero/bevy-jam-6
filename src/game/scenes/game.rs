use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::camera::{CameraTarget, MainCamera};
use crate::game::effects::lightning_ball::{LightningBall, LightningBallConduit};
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::enemy::Enemy;
use crate::game::prefabs::game_world::GameWorld;
use crate::game::prefabs::game_world_markers::{
    EntityWithGlobalTransformQueryData, GameWorldMarkerSystemParam, TempleLight, TempleRoof,
    auto_collider_mesh_obs,
};
use crate::game::prefabs::player::{Player, PlayerSystemParam};
use crate::game::screens::Screen;
use crate::game::utils::extensions::vec2::Vec2Ext;
use avian3d::prelude::{ExternalAngularImpulse, ExternalImpulse, Friction, Mass};
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_auto_plugin::auto_plugin::*;
use smart_default::SmartDefault;
use std::ops::{Deref, DerefMut};

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[require(Visibility)]
pub struct LevelRoot;

pub fn spawn_level(mut commands: Commands) {
    info!("spawning world");
    commands
        .spawn((GameWorld, StateScoped(Screen::Gameplay)))
        .observe(auto_collider_mesh_obs)
        .observe(spawn_extras_on_instance_ready)
        .with_child((
            Name::new("Sun"),
            DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            CascadeShadowConfigBuilder {
                maximum_distance: 99999.9,
                ..Default::default()
            }
            .build(),
            Transform::from_translation(Vec3::Y * 100.0)
                .with_rotation(Quat::from_rotation_x(-45_f32.to_radians())),
        ));
}

fn spawn_extras_on_instance_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    mut game_world_marker: GameWorldMarkerSystemParam,
) {
    info!("Trigger<SceneInstanceReady>");
    commands.entity(trigger.observer()).despawn();
    info!("spawning player");
    let player = game_world_marker.spawn_in_player_spawn(Player, None);
    info!("spawning enemies");
    for pos in generate_pin_layout(5.0, 0.5, 3, Facing::Toward) {
        game_world_marker.spawn_in_enemy_spawn(
            (
                Enemy::BaseSkele,
                LightningBallConduit,
                Mass(1.0),
                Friction::new(0.4),
                TargetEnt {
                    // TODO: spawn point doesnt work?
                    target_ent: player, // game_world_marker.player_spawn.entity,
                    within_distance: 10.0,
                },
            ),
            Some(Transform::from_scale(Vec3::splat(4.0)).with_translation(pos.to_vec3())),
        );
    }
}

#[derive(Debug, SmartDefault)]
struct DemoCache {
    #[default = 1.0]
    power: f32,
    #[default = 0.0]
    accuracy: f32,
    #[default = 1.0]
    turn_rate: f32,
}
fn demo_input(
    time: Res<Time>,
    mut commands: Commands,
    mut local: Local<(bool, DemoCache)>,
    mut player_system_param: PlayerSystemParam,
    button_input: Res<ButtonInput<KeyCode>>,
) {
    let mut apply_transform = |transform: Transform| {
        commands
            .entity(player_system_param.entity())
            .insert(transform);
    };
    let (changed, cache) = &mut *local;
    let max_accuracy_offset: f32 = 30_f32.to_radians();
    if button_input.pressed(KeyCode::ArrowLeft) {
        cache.accuracy += 1_f32.to_radians();
        cache.accuracy = cache
            .accuracy
            .clamp(-max_accuracy_offset, max_accuracy_offset);
        *changed = true;
    }
    if button_input.pressed(KeyCode::ArrowRight) {
        cache.accuracy -= 1_f32.to_radians();
        cache.accuracy = cache
            .accuracy
            .clamp(-max_accuracy_offset, max_accuracy_offset);
        *changed = true;
    }
    if button_input.pressed(KeyCode::ArrowUp) {
        cache.power += 0.1;
        *changed = true;
    }
    if button_input.pressed(KeyCode::ArrowDown) {
        cache.power -= 0.1;
        *changed = true;
    }
    if button_input.pressed(KeyCode::KeyW) {
        cache.turn_rate += 1.0;
        cache.turn_rate = cache.turn_rate.max(1.0);
        *changed = true;
    }
    if button_input.pressed(KeyCode::KeyS) {
        cache.turn_rate -= 1.0;
        cache.turn_rate = cache.turn_rate.max(1.0);
        *changed = true;
    }
    if button_input.pressed(KeyCode::KeyA) {
        let mut transform = player_system_param.player_transform.clone();
        transform.rotate(Quat::from_rotation_y(
            1_f32.to_radians() * cache.turn_rate * time.delta_secs(),
        ));
        apply_transform(transform);
        *changed = true;
    }
    if button_input.pressed(KeyCode::KeyD) {
        let mut transform = player_system_param.player_transform.clone();
        transform.rotate(Quat::from_rotation_y(
            -1_f32.to_radians() * cache.turn_rate * time.delta_secs(),
        ));
        apply_transform(transform);
        *changed = true;
    }
    if *changed {
        *changed = false;
        info!(
            "demo value updated: {cache:?} player_rotation: {}",
            player_system_param.player_transform.rotation
        );
    }
    if button_input.just_pressed(KeyCode::Space) {
        player_system_param.spawn_bowling_ball(cache.power, cache.accuracy);
    }
}

// TODO: move
fn get_pitch_and_roll(quat: Quat) -> (f32, f32) {
    // Local forward and right vectors
    let local_forward = Vec3::Z;
    let local_right = Vec3::X;

    // Transform to world space
    let world_forward = quat * local_forward;
    let world_right = quat * local_right;

    // Pitch: angle between forward vector and horizontal plane (XZ)
    let pitch = world_forward
        .y
        .atan2((world_forward.x.powi(2) + world_forward.z.powi(2)).sqrt());

    // Roll: angle between right vector and horizontal plane (YZ)
    let roll = world_right
        .y
        .atan2((world_right.x.powi(2) + world_right.z.powi(2)).sqrt());

    (pitch, roll)
}

fn hide_roof(
    mut commands: Commands,
    roof: Single<(Entity, &Visibility), With<TempleRoof>>,
    light: Single<(Entity, &Visibility), With<TempleLight>>,
    main_camera: Single<EntityWithGlobalTransformQueryData, With<MainCamera>>,
) {
    let (pitch, roll) = get_pitch_and_roll(main_camera.global_transform.rotation());
    let past_threshold = pitch >= 0.24;
    let mut toggle_visibility = |(entity, visibility): (Entity, &Visibility)| {
        if past_threshold && matches!(visibility, Visibility::Visible | Visibility::Inherited) {
            info!("hiding {entity}");
            commands.entity(entity).insert(Visibility::Hidden);
        } else if !past_threshold && matches!(visibility, Visibility::Hidden) {
            info!("revealing {entity}");
            commands.entity(entity).insert(Visibility::Inherited);
        }
    };
    toggle_visibility(*roof);
    toggle_visibility(*light);
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, demo_input);
    app.add_systems(Update, hide_roof);
}

#[derive(Debug, Clone, Copy)]
pub enum Facing {
    Away,
    Toward,
}
pub fn generate_pin_layout(pin_width: f32, spacing: f32, rows: usize, facing: Facing) -> Vec<Vec2> {
    let mut positions = Vec::new();
    for r in 0..rows {
        let num_in_row = (rows - r) as f32;
        let y = (r as f32) * (pin_width + spacing);
        // total width occupied by this row: N * pin_width + (N - 1) * spacing
        let row_width = num_in_row * pin_width + (num_in_row - 1.0) * spacing;

        // The first pin’s center x should be at:
        //   -row_width/2 + pin_width/2
        // so that the row is centered around x = 0.0
        let start_x = -row_width / 2.0 + pin_width / 2.0;

        for i in 0..(num_in_row as usize) {
            let x = start_x + (i as f32) * (pin_width + spacing);
            let y = match facing {
                Facing::Away => -y,
                Facing::Toward => y,
            };
            positions.push(Vec2::new(x, y));
        }
    }

    positions
}
