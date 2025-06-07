use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

pub mod break_down_gltf;
pub mod lightning_ball;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(lightning_ball::plugin);
    app.add_plugins(break_down_gltf::plugin);
}
