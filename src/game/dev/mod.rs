mod fps;
mod inspector_ui;
mod selection;

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use smart_default::SmartDefault;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Copy, Clone, SmartDefault, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct DebugUiEnabled(#[default(false)] bool);

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(fps::plugin);
    #[cfg(feature = "inspector_ui")]
    {
        app.add_plugins(inspector_ui::plugin);
        app.add_plugins(selection::plugin);
    }
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(KeyCode::Backquote)),
    );
}

fn toggle_debug_ui(
    mut debug_ui_enabled: ResMut<DebugUiEnabled>,
    mut debug_bevy_ui_enabled: ResMut<UiDebugOptions>,
) {
    // local debug ui
    debug_ui_enabled.0 = !debug_ui_enabled.0;

    // bevy ui
    debug_bevy_ui_enabled.toggle();
}
