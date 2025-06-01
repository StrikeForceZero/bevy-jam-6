pub mod level;
mod tower;
mod wizard;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(tower::plugin);
    app.add_plugins(wizard::plugin);
    app.add_plugins(level::plugin);
}
