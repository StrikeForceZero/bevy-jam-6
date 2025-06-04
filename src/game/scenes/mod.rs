use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

pub mod simple_bowling;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(simple_bowling::plugin);
}
