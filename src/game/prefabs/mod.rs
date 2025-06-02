pub mod enemy;
pub mod tower;
pub mod wizard;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(enemy::plugin);
    app.add_plugins(tower::plugin);
    app.add_plugins(wizard::plugin);
}
