use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    text::TextColor,
    ui::{AlignItems, JustifyContent, Node, UiRect, Val},
};
use bevy_auto_plugin::auto_plugin::*;

use crate::game::scenes::LevelData;

#[derive(Component, Clone, Copy)]
pub struct TempleHealthUi;

#[derive(Component, Clone, Copy)]
pub struct BallThrowsLeftUI;

#[derive(Component, Clone, Copy)]
pub struct KillCountUI;

fn temple_health(asset_server: &AssetServer) -> impl Bundle + use<> {
    let temple_image = asset_server.load_with_settings(
        "images/temple.png",
        |settings: &mut ImageLoaderSettings| {
            // Need to use nearest filtering to avoid bleeding between the slices with tiling
            settings.sampler = ImageSampler::nearest();
        },
    );
    children![
        ImageNode {
            image: temple_image.clone(),
            ..default()
        },
        (
            Text::new(""),
            TextColor::WHITE,
            BorderColor(Color::BLACK),
            TempleHealthUi,
        ),
    ]
}

fn update_temple_health(
    level_data: Res<LevelData>,
    temple_text: Single<&mut Text, With<TempleHealthUi>>,
) {
    let mut text = temple_text.into_inner();
    *text = Text::new(format!(" {}", level_data.temple_health));
}

fn spawn_hud_elements(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Px(600.0),
            height: Val::Px(200.0),
            left: Val::Px(20.0),
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(20.0)),
            align_items: AlignItems::FlexStart,
            ..default()
        },
        temple_health(&asset_server),
    ));
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_hud_elements);
    app.add_systems(Update, update_temple_health);
}
