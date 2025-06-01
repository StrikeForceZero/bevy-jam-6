//! Spawn the main level.

use bevy::{
    color::palettes::css::{RED, YELLOW},
    prelude::*,
};
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{
    camera::CameraTarget,
    despawn::DespawnDelayed,
    health::{Dead, Health},
    spark::{Spark, SparkTarget},
};

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Transform::default(), CameraTarget));

    let enemy_mesh = meshes.add(Circle::new(10.0));
    let enemy_material = materials.add(ColorMaterial::from_color(RED));

    const POSITIONS: [Vec2; 6] = [
        Vec2::new(0.0, 0.0),
        Vec2::new(50.0, 30.0),
        Vec2::new(50.0, -70.0),
        Vec2::new(100.0, 40.0),
        Vec2::new(100.0, 50.0),
        Vec2::new(100.0, 60.0),
    ];

    let batch: Vec<_> = POSITIONS
        .iter()
        .map(|pos| {
            (
                Name::new("Demo Enemy"),
                Transform::from_translation(pos.extend(0.0)),
                Mesh2d(enemy_mesh.clone()),
                MeshMaterial2d(enemy_material.clone()),
                SparkTarget,
                Health(50.0),
            )
        })
        .collect();

    commands.spawn_batch(batch);
}

fn handle_spark_target_died(
    tr: Trigger<OnInsert, Dead>,
    q: Query<(), With<SparkTarget>>,
    mut commands: Commands,
) {
    if !q.contains(tr.target()) {
        return;
    }

    commands.entity(tr.target()).trigger(DespawnDelayed);
}

fn handle_key_down(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        commands.spawn((
            Transform::from_xyz(0.0, 0.0, 10.0),
            Mesh2d(meshes.add(Circle::new(5.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(YELLOW))),
            Spark,
        ));
    }
}

#[auto_plugin(app=app)]
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, handle_key_down)
        .add_observer(handle_spark_target_died);
}
