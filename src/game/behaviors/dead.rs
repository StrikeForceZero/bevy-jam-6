use crate::game::behaviors::despawn_at::DespawnAt;
use bevy::ecs::component::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct Dead;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
#[require(Dead)]
/// Dead at [`Res<Time>`].elapsed_secs()
pub struct DeadAt(pub f32);

impl DeadAt {
    // TODO: maybe we should move this hook to Dead like Stunned
    /// Automatically sets the [`DeadAt`] time to now if it was inserted with the default value
    fn on_insert(mut world: DeferredWorld, hook_context: HookContext) {
        let &DeadAt(secs) = world
            .entity(hook_context.entity)
            .get::<Self>()
            .expect("DeadAt::on_insert failed to resolve query - impossible");
        if secs != 0.0 {
            return;
        }
        let elapsed = world.resource::<Time>().elapsed_secs();
        world
            .commands()
            .entity(hook_context.entity)
            .insert(DeadAt(elapsed));
    }
}

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(Dead)]
#[require(DeadAt)]
/// Dead and Despawn in [`Res<Time>`].elapsed_secs() >= [`DeadAt`] + secs
pub struct DeadAndDespawnIn(pub f32);

fn on_insert_dead_and_despawn_in(
    trigger: Trigger<OnInsert, DeadAndDespawnIn>,
    entity_q: Query<DeadQueryData, With<Dead>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    debug!("Trigger<OnInsert, DeadAndDespawnIn> {entity}");
    let item = entity_q
        .get(entity)
        .expect("Trigger<OnInsert, DeadAndDespawnIn> failed to resolve query - impossible");
    // TODO: this was a foot gun, apparently require(DeadAt) doesn't run its hook until after this hook..
    let dead_at = if item.dead_at.0 == 0.0 {
        time.elapsed_secs()
    } else {
        item.dead_at.0
    };
    let expected_despawn_at = dead_at
        + item
            .dead_and_despawn_in_opt
            .as_ref()
            .expect("Trigger<OnInsert, DeadAndDespawnIn> failed to resolve query - impossible")
            .0;
    let mut entity_cmds = commands.entity(entity);
    if let Some(despawn_at) = &item.despawn_at_opt {
        if despawn_at.0 == expected_despawn_at {
            return;
        }
    };
    trace!(
        "updating despawn_at: {:?} -> {:?}",
        item.despawn_at_opt,
        DespawnAt(expected_despawn_at)
    );
    entity_cmds.insert(DespawnAt(expected_despawn_at));
}

#[derive(QueryData)]
pub struct DeadQueryData {
    pub entity: Entity,
    pub dead: Ref<'static, Dead>,
    pub dead_at: Ref<'static, DeadAt>,
    pub dead_and_despawn_in_opt: Option<Ref<'static, DeadAndDespawnIn>>,
    pub despawn_at_opt: Option<Ref<'static, DespawnAt>>,
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_insert_dead_and_despawn_in);
}
