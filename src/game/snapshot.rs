use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Snapshot<T>(Option<T>);

impl<T> Snapshot<T> {
    pub fn replace(&mut self, new: T) -> Option<T> {
        self.0.replace(new)
    }
}
#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}
