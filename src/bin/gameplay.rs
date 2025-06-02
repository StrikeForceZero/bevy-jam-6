use bevy::prelude::*;
use bevy_game_jam_6::game::GamePlugin;
use bevy_game_jam_6::game::screens::Screen;

fn main() {
    let mut app = App::new();
    app.add_plugins(GamePlugin);
    app.add_systems(PostStartup, |mut nex_screen: ResMut<NextState<Screen>>| {
        nex_screen.set(Screen::Loading);
    });
    app.run();
}
