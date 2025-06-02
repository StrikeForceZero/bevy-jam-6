pub mod behaviors;
mod enemy;
pub mod level;
mod lightning_ball;
mod spawner;
mod tower;
mod wizard;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(behaviors::plugin);
    app.add_plugins(enemy::plugin);
    app.add_plugins(lightning_ball::plugin);
    app.add_plugins(level::plugin);
    app.add_plugins(spawner::plugin);
    app.add_plugins(tower::plugin);
    app.add_plugins(wizard::plugin);
}
