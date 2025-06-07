//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{asset_tracking::ResourceHandles, screens::Screen, theme::prelude::*};

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        StateScoped(Screen::Loading),
        children![widget::label("Loading...")],
    ));
}

fn enter_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

pub fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}

#[auto_plugin(app=app)]
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);

    app.add_systems(
        Update,
        enter_gameplay_screen.run_if(in_state(Screen::Loading).and(all_assets_loaded)),
    );
}
