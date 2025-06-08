use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{menus::Menu, scenes::LevelData, screens::Screen, theme::widget};

fn spawn_end_menu(mut commands: Commands, ld: Res<LevelData>) {
    commands.spawn((
        widget::ui_root("End"),
        GlobalZIndex(2),
        StateScoped(Menu::End),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Text::new(format!("Zeus sent {} skeles back to Hades!", ld.kill_count)),
                TextFont::from_font_size(30.),
                TextColor(Color::srgb(0.4, 0.769, 1.)),
            ),
            (
                Text::new("Can he do better?"),
                TextFont::from_font_size(30.),
                TextColor(Color::srgb(0.7, 0.769, 0.9)),
            ),
            widget::button("Play Again?", play_again),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn play_again(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut ld: ResMut<LevelData>,
) {
    *ld = LevelData::default();
    next_screen.set(Screen::Gameplay);
}

fn quit_to_title(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut ld: ResMut<LevelData>,
) {
    *ld = LevelData::default();
    next_screen.set(Screen::Title);
}

#[auto_plugin(app=app)]
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::End), spawn_end_menu);
}
