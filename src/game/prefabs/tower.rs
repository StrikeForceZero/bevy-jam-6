use avian3d::prelude::{Collider, RigidBody};
use bevy::color::palettes::css::GRAY;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct Tower;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_tower_added);
}

fn on_tower_added(
    trigger: Trigger<OnAdd, Tower>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const RADIUS: f32 = 10.0;
    const HEIGHT: f32 = 100.0;
    commands.entity(trigger.target()).insert((
        Mesh3d(meshes.add(Cylinder::new(RADIUS, HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::from(GRAY),
            perceptual_roughness: 1.0,
            ..Default::default()
        })),
        Collider::cylinder(RADIUS, HEIGHT),
        RigidBody::Static,
    ));
}
