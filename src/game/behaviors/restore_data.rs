use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[derive(Component, Debug, Default, Copy, Clone)]
#[component(immutable)]
pub struct OriginalData<T>(pub T)
where
    T: Component + Clone;

impl<T> OriginalData<T>
where
    T: Component + Clone,
{
    pub fn restore(&self, entity_commands: &mut EntityCommands) {
        entity_commands
            .insert(self.0.clone())
            .remove::<OriginalData<T>>();
    }
}

#[derive(QueryData)]
pub struct RestorableQueryData<T>
where
    T: Component + Clone,
{
    current: Option<&'static T>,
    original: Option<&'static OriginalData<T>>,
}

impl<T> RestorableQueryDataItem<'_, T>
where
    T: Component + Clone,
{
    pub fn restore(&self, entity_commands: &mut EntityCommands) {
        if let Some(original) = self.original {
            original.restore(entity_commands);
        }
    }
    pub fn store(&self, entity_commands: &mut EntityCommands) {
        if let Some(current) = self.current {
            entity_commands.insert(OriginalData(current.clone()));
        }
    }
    pub fn remove(&self, entity_commands: &mut EntityCommands) {
        entity_commands.remove::<T>();
    }
    pub fn store_and_remove(&self, entity_commands: &mut EntityCommands) {
        self.store(entity_commands);
        self.remove(entity_commands);
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}
