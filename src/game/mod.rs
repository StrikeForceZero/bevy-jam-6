#[macro_use]
mod enforce_exists;

mod asset_tracking;
mod audio;
pub mod behaviors;
mod camera;
mod constants;
mod despawn;
#[cfg(feature = "dev")]
mod dev;
mod effects;
mod game_system_set;
mod health;
mod menus;
mod pause_controller;
mod physics;
mod prefabs;
mod rng;
mod scenes;
pub mod screens;
mod snapshot;
mod theme;
mod utils;

use crate::game::rng::RngPlugin;
use bevy::app::PluginGroupBuilder;
use bevy::asset::AssetMetaCheck;
use bevy::asset::ron::extensions;
#[cfg(feature = "dev_frame_count_log")]
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use bevy_egui::EguiPlugin;
#[cfg(feature = "dev_frame_count_log")]
use bevy_frame_count_log_prefix::prelude::FrameCountLogPrefixPlugin;
use bevy_hanabi::HanabiPlugin;
use bevy_skein::SkeinPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    #[auto_plugin(app=app)]
    fn build(&self, app: &mut App) {
        // Bevy
        app.add_plugins(default_plugins());
        app.add_plugins(MeshPickingPlugin);

        // External
        #[cfg(feature = "dev_frame_count_log")]
        app.add_plugins(FrameCountLogPrefixPlugin);
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        });
        app.add_plugins(HanabiPlugin);
        app.add_plugins(SkeinPlugin::default());

        // Internal
        app.add_plugins(RngPlugin);
        app.add_plugins(game_system_set::plugin);
        app.add_plugins(utils::plugin);
        app.add_plugins(camera::plugin);
        #[cfg(feature = "dev")]
        app.add_plugins(dev::plugin);
        app.add_plugins(asset_tracking::plugin);
        app.add_plugins(pause_controller::plugin);
        app.add_plugins(physics::plugin);
        app.add_plugins(snapshot::plugin);
        app.add_plugins(behaviors::plugin);
        app.add_plugins(effects::plugin);
        app.add_plugins(prefabs::plugin);
        app.add_plugins(scenes::plugin);
        app.add_plugins(audio::plugin);
        app.add_plugins(theme::plugin);
        app.add_plugins(menus::plugin);
        app.add_plugins(screens::plugin);
        app.add_plugins(health::plugin);
        app.add_plugins(despawn::plugin::<PreUpdate>);
    }
}

fn default_plugins() -> PluginGroupBuilder {
    let default_plugins = DefaultPlugins
        .build()
        // TODO: should we conditionally disable this based on wasm feature?
        .set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics on web build on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Window {
                title: "Bevy Jam 6".to_string(),
                fit_canvas_to_parent: true,
                ..default()
            }
            .into(),
            ..default()
        });
    #[cfg(feature = "dev_frame_count_log")]
    let default_plugins = default_plugins.disable::<LogPlugin>();
    default_plugins
}
