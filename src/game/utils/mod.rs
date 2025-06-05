pub mod extensions;
pub mod vector;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(extensions::plugin);
    app.add_plugins(vector::plugin);
}
