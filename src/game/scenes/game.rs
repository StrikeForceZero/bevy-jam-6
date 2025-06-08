use crate::game::behaviors::pin_joint::{OnAddPinJoint, PinJoint, PinJoints};
use crate::game::behaviors::spawn_group::{SpawnGroup, SpawnGroupItem};
use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::camera::{CameraTarget, MainCamera};
use crate::game::effects::lightning_ball::{LightningBall, LightningBallConduit};
use crate::game::prefabs::bowling_ball::BowlingBall;
use crate::game::prefabs::enemy::{Enemy, SKELE_WIDTH};
use crate::game::prefabs::game_world::GameWorld;
use crate::game::prefabs::game_world_markers::{
    EntityWithGlobalTransformQueryData, GameWorldMarkerSystemParam, TempleLight, TempleRoof,
    auto_collider_mesh_obs,
};
use crate::game::prefabs::player::{Player, PlayerSystemParam};
use crate::game::screens::Screen;
use crate::game::utils::extensions::vec2::Vec2Ext;
use crate::game::utils::quat;
use avian3d::prelude::{
    DistanceJoint, ExternalAngularImpulse, ExternalImpulse, FixedJoint, Friction, Joint, Mass,
    SphericalJoint,
};
use bevy::ecs::entity::EntityHashMap;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_auto_plugin::auto_plugin::*;
use itertools::Itertools;
use smart_default::SmartDefault;
use std::ops::{Deref, DerefMut};

use super::LevelData;

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
    for ix in 0..1 {
        let formation_id = game_world_marker.spawn_in_enemy_spawn(
            (Name::new(format!("SkeleGroup({ix})")), SpawnGroup(ix)),
            None,
        );
        let (layout, layout_entries) = generate_pin_layout(SKELE_WIDTH, 0.5, 5, Facing::Toward);
        let pin_entity_layout_tuples = layout_entries
            .into_iter()
            .map(|entry| {
                let pin_id = commands
                    .spawn((
                        ChildOf(formation_id),
                        SpawnGroupItem(formation_id),
                        Enemy::BaseSkele,
                        LightningBallConduit,
                        Mass(1.0),
                        Friction::new(0.4),
                        PinJoints::default(),
                        TargetEnt {
                            target_ent: game_world_marker.player_spawn.target_entity(),
                            within_distance: 10.0,
                        },
                        Transform::from_scale(Vec3::splat(4.0))
                            .with_translation(entry.pos.to_vec3()),
                    ))
                    .id();
                let pin = Pin { entity: pin_id };
                (pin, entry)
            })
            .collect_vec();
        let pin_pos_map = pin_entity_layout_tuples
            .iter()
            .map(|&(pin, entry)| (pin.entity, entry.pos))
            .collect::<EntityHashMap<_>>();
        let rest = layout.spacing + SKELE_WIDTH * 20.0;
        for (a, b) in generate_pin_joints(&pin_entity_layout_tuples) {
            let pin_joint_id = commands
                .spawn((
                    Name::new(format!("Joint({a}, {b})")),
                    ChildOf(formation_id),
                    PinJoint::new(a, b),
                    // anchors default to (0,0) on each body
                    SphericalJoint::new(a, b)
                        .with_local_anchor_1(pin_pos_map[&a].to_vec3())
                        .with_local_anchor_2(pin_pos_map[&b].to_vec3())
                        .with_point_compliance(1.0 / 100.0)
                        .with_swing_compliance(0.0)
                        .with_twist_compliance(0.0)
                        .with_linear_velocity_damping(0.1) // kill any tiny residual drift
                        .with_angular_velocity_damping(0.1), // optional, to damp spins
                ))
                .id();
            for entity in [a, b] {
                commands.entity(entity).trigger(OnAddPinJoint(pin_joint_id));
            }
        }
    }
}

#[derive(Debug, SmartDefault)]
struct DemoCache {
    #[default = 1.0]
    power: f32,
    #[default = 0.0]
    accuracy: f32,
    #[default = 30.0]
    turn_rate: f32,
}
fn demo_input(
    time: Res<Time>,
    mut commands: Commands,
    mut local: Local<(bool, DemoCache)>,
    mut player_system_param: PlayerSystemParam,
    mut level_data: ResMut<LevelData>,
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
        if level_data.balls_left > 0 {
            player_system_param.spawn_bowling_ball(cache.power, cache.accuracy);
            level_data.balls_left -= 1;
        }
    }
}

fn hide_roof(
    mut commands: Commands,
    roof: Single<(Entity, &Visibility), With<TempleRoof>>,
    light: Single<(Entity, &Visibility), With<TempleLight>>,
    main_camera: Single<EntityWithGlobalTransformQueryData, With<MainCamera>>,
) {
    let (pitch, _roll) = quat::get_pitch_and_roll(main_camera.global_transform.rotation());
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

#[derive(Debug, Clone, Copy)]
struct Pin {
    entity: Entity,
}

#[derive(Debug, Clone, Copy)]
struct PinLayoutEntry {
    pos: Vec2,
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy)]
struct PinLayout {
    pin_count: usize,
    spacing: f32,
}

pub fn generate_pin_layout(
    pin_width: f32,
    spacing: f32,
    rows: usize,
    facing: Facing,
) -> (PinLayout, Vec<PinLayoutEntry>) {
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
            positions.push(PinLayoutEntry {
                pos: Vec2::new(x, y),
                row: r,
                col: i,
            });
        }
    }

    let pin_layout = PinLayout {
        pin_count: positions.len(),
        spacing: pin_width * spacing,
    };

    (pin_layout, positions)
}

/// Given a list of (position, row, col), return all unique joints
/// 1. its right neighbor in the same row (r, c+1)
/// 2. its down-left neighbor in the next row (r+1, c-1)
/// 3. its down-right neighbor in the next row (r+1, c)
pub fn generate_pin_joints(layout: &[(Pin, PinLayoutEntry)]) -> Vec<(Entity, Entity)> {
    // Build a lookup table: (row, col) → Entity
    let mut lookup: HashMap<(usize, usize), Entity> = HashMap::new();
    for &(pin, pin_layout) in layout.iter() {
        lookup.insert((pin_layout.row, pin_layout.col), pin.entity);
    }

    let mut joints = Vec::new();

    // For each pin, connect to right, down-left, and down-right neighbors
    for &(pin, pin_layout) in layout.iter() {
        let (r, c) = (pin_layout.row, pin_layout.col);
        let a = pin.entity;

        // Right neighbor: same row, col + 1
        if let Some(&b) = lookup.get(&(r, c + 1)) {
            joints.push((a, b));
        }
        // Down-left neighbor: row + 1, col - 1
        if c > 0 {
            if let Some(&b) = lookup.get(&(r + 1, c - 1)) {
                joints.push((a, b));
            }
        }
        // Down-right neighbor: row + 1, same col
        if let Some(&b) = lookup.get(&(r + 1, c)) {
            joints.push((a, b));
        }
    }

    joints
}
