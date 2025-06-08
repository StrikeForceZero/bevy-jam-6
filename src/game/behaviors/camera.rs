use crate::game::camera::CameraTarget;
use crate::game::pause_controller::Pause;
use crate::game::prefabs::player::Player;
use crate::game::screens::Screen;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

fn camera_target(
    mut commands: Commands,
    button_input: Res<ButtonInput<KeyCode>>,
    player: Query<Entity, With<Player>>,
) {
    // If space is not being held, retarget to Player
    if !button_input.pressed(KeyCode::Space) {
        if let Some(ent) = player.iter().next() {
            commands.entity(ent).insert(CameraTarget);
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        camera_target.run_if(in_state(Pause(false)).and(in_state(Screen::Gameplay))),
    );
}
