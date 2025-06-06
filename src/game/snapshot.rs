use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Snapshot<T>(pub Option<T>);

impl<T> Default for Snapshot<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Snapshot<T> {
    pub fn replace(&mut self, new: T) -> Option<T> {
        self.0.replace(new)
    }
}
#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}
