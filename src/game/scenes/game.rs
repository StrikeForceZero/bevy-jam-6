use crate::game::behaviors::target_ent::TargetEnt;
use crate::game::camera::CameraTarget;
use crate::game::effects::lightning_ball::{LightningBall, LightningBallConduit};
use crate::game::prefabs::enemy::Enemy;
use crate::game::prefabs::tower::Tower;
use crate::game::prefabs::wizard::Wizard;
use crate::game::screens::Screen;
use avian3d::prelude::Collider;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=_app)]
pub fn plugin(_app: &mut App) {}

pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let level_ent = commands
        .spawn((
            Name::new("Level"),
            StateScoped(Screen::Gameplay),
            Transform::default(),
            Visibility::default(),
            children![
                (
                    Name::new("Grass"),
                    Mesh3d(meshes.add(Cuboid::new(1000.0, 10.0, 1000.0))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::from(GREEN),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    })),
                ),
                (
                    LightningBall,
                    CameraTarget,
                    Transform::from_xyz(0.0, 3.1 * 10.0 + 100.0, 0.8 * 10.0),
                ),
            ],
        ))
        .id();

    let tower_ent = commands
        .entity(level_ent)
        .with_child((
            Tower,
            Transform::from_xyz(0.0, 50.0, 0.0),
            children![(
                Wizard,
                Transform::from_xyz(0.0, 50.0, 0.0).with_scale(Vec3::splat(10.0)),
                children![(
                    Name::new("Fake Staff Pos"),
                    LightningBallConduit,
                    Transform::from_xyz(-0.81, 1.95, -0.09),
                    Collider::sphere(0.25)
                )],
            ),],
        ))
        .id();

    commands.entity(level_ent).with_child((
        Name::new("Skele"),
        LightningBallConduit,
        Enemy::BaseSkele,
        Transform::from_xyz(100.0, 10.0, 100.0).with_scale(Vec3::splat(15.0)),
        TargetEnt {
            target_ent: tower_ent,
            within_distance: 20.0,
        },
    ));
}
