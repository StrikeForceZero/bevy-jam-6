use crate::game::camera::CameraTarget;
use crate::game::screens::Screen;
use crate::game::tower_defense::lightning_ball::LightningBall;
use crate::game::tower_defense::tower::Tower;
use crate::game::tower_defense::wizard::Wizard;
use bevy::color::palettes::css::GREEN;
use bevy::prelude::*;
use bevy::ui::OverflowAxis::Visible;
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
                Mesh3d(meshes.add(Cuboid::new(1000.0, 10.0, 1000.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::from(GREEN),
                    perceptual_roughness: 1.0,
                    reflectance: 0.0,
                    ..Default::default()
                })),
            ),
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
                LightningBall,
                Transform::from_xyz(0.0, 3.1 * 10.0 + 100.0, 0.8 * 10.0),
            )
        ],
    ));
}
