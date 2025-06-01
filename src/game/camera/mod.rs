use bevy::core_pipeline::bloom::Bloom;
use bevy::pbr::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
#[require(PanOrbitCamera)]
#[require(ShadowFilteringMethod::Hardware2x2)]
pub struct MainCamera;

#[auto_register_type]
#[derive(Component, Debug, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
#[require(Transform)]
pub struct CameraTarget;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PanOrbitCameraPlugin);
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, update_camera_target);
    app.insert_resource(AmbientLight {
        brightness: 2.0,
        ..Default::default()
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera {
            hdr: true,
            ..Default::default()
        },
        Bloom::NATURAL,
        PanOrbitCamera {
            radius: Some(400.0),
            focus: Vec3::ZERO,
            ..Default::default()
        },
    ));
}

fn update_camera_target(
    mut pan_orbit_q: Single<Mut<PanOrbitCamera>>,
    target_q: Single<
        &GlobalTransform,
        Or<(
            Added<CameraTarget>,
            (With<CameraTarget>, Changed<GlobalTransform>),
        )>,
    >,
) {
    if pan_orbit_q.focus == target_q.translation() {
        return;
    }
    pan_orbit_q.target_focus = target_q.translation();
    // Whenever changing properties manually like this, it's necessary to force
    // PanOrbitCamera to update this frame (by default it only updates when there are
    // input events).
    pan_orbit_q.force_update = true;
}
