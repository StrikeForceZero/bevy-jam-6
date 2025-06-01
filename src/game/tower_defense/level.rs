use crate::game::camera::CameraTarget;
use crate::game::screens::Screen;
use crate::game::tower_defense::tower::Tower;
use crate::game::tower_defense::wizard::Wizard;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}

pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Level"),
        StateScoped(Screen::Gameplay),
        Transform::default(),
        Visibility::default(),
        children![
            (Tower, Transform::from_xyz(0.0, 50.0, 0.0)),
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
                Wizard,
                CameraTarget,
                Transform::from_xyz(0.0, 100.0, 0.0).with_scale(Vec3::splat(10.0)),
            ),
            (
                PointLight {
                    intensity: 19999999999.0,
                    range: 100000.0,
                    radius: 999.0,
                    shadows_enabled: true,
                    ..Default::default()
                },
                Transform::from_xyz(30.0, 300.0, 80.0),
            )
        ],
    ));
}
