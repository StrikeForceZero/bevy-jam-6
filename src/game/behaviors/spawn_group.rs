use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(SpawnGroupItems)]
pub struct SpawnGroup(pub usize);

#[auto_register_type]
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = SpawnGroupItem, linked_spawn)]
pub struct SpawnGroupItems(Vec<Entity>);

impl SpawnGroupItems {
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = SpawnGroupItems)]
pub struct SpawnGroupItem(pub Entity);

#[derive(QueryData)]
pub struct SpawnGroupQueryData {
    pub entity: Entity,
    pub spawn_group: &'static SpawnGroup,
    pub spawn_group_items: &'static SpawnGroupItems,
}

#[derive(QueryData)]
pub struct SpawnGroupItemQueryData {
    pub entity: Entity,
    pub spawn_group_item: &'static SpawnGroupItem,
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {}
