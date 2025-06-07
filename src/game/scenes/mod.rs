use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use smart_default::SmartDefault;

pub mod game;
pub mod ui;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, SmartDefault, Debug, Clone, Reflect)]
pub struct LevelData {
    #[default(20)]
    pub temple_health: usize,
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(game::plugin);
    app.add_plugins(ui::plugin);
}
