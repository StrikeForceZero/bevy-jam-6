use crate::game::pause_controller::Pause;
use avian3d::prelude::{
    Physics, PhysicsInterpolationPlugin, PhysicsPickingPlugin, PhysicsPlugins, PhysicsTime,
};
use avian3d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Resource)]
struct PhysicsDebugGizmosEnabled(bool);

fn toggle_gizmos(
    mut gizmos: ResMut<GizmoConfigStore>,
    mut debug_gizmos_enabled: ResMut<PhysicsDebugGizmosEnabled>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyV) {
        debug_gizmos_enabled.0 = !debug_gizmos_enabled.0;
    }
    if !debug_gizmos_enabled.is_changed() {
        return;
    }
    gizmos.config_mut::<PhysicsGizmos>().0.enabled = debug_gizmos_enabled.0;
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().set(PhysicsInterpolationPlugin::extrapolate_all()));
    app.add_plugins(PhysicsPickingPlugin);
    app.add_plugins(PhysicsDebugPlugin::default());
    app.world_mut()
        .resource_mut::<GizmoConfigStore>()
        .config_mut::<PhysicsGizmos>()
        .0
        .enabled = app.world().resource::<PhysicsDebugGizmosEnabled>().0;
    app.add_systems(OnEnter(Pause(false)), |mut time: ResMut<Time<Physics>>| {
        time.unpause();
    });
    app.add_systems(OnEnter(Pause(true)), |mut time: ResMut<Time<Physics>>| {
        time.pause();
    });
    app.add_systems(Update, toggle_gizmos);
}
