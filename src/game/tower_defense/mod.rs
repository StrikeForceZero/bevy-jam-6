pub mod level;
mod tower;
mod wizard;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    tower::plugin(app);
    wizard::plugin(app);
    level::plugin(app);
}
