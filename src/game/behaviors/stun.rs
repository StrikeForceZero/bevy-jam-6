use crate::game::behaviors::dead::Dead;
use crate::game::behaviors::grounded::{Grounded, GroundedSystemParam};
use bevy::ecs::component::HookContext;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use std::fmt::Debug;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[component(on_insert=Self::on_insert,on_remove=Self::on_remove)]
pub struct Stunned;

impl Stunned {
    /// Automatically sets the [`DeadAtElapsedSecs`] time to now if it was inserted with the default value
    fn on_insert(mut world: DeferredWorld, hook_context: HookContext) {
        let StunnedAt(secs) = world
            .entity(hook_context.entity)
            .get::<StunnedAt>()
            .copied()
            .unwrap_or_default();
        if secs != 0.0 {
            return;
        }
        let elapsed = world.resource::<Time>().elapsed_secs();
        world
            .commands()
            .entity(hook_context.entity)
            .insert(StunnedAt(elapsed));
    }
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .try_remove::<StunnedAt>()
            .try_remove::<StunTime>();
    }
}

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Stunned)]
/// Stunned at [`Res<Time>`].elapsed_secs()
pub struct StunnedAt(pub f32);

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Stunned)]
/// Number of secs to remain stunned for
pub struct StunTime(pub f32);

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Stunned)]
/// Only allows unstun when an entity is Grounded
pub struct UnStunOnlyAllowedWhenGrounded;

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Default, Copy, Clone, Reflect)]
pub struct OnStunned;

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Default, Copy, Clone, Reflect)]
pub struct OnUnStunned;

#[derive(QueryData)]
struct StunnedAtQueryData {
    entity: Entity,
    stunned_at: &'static StunnedAt,
    stun_time: Option<&'static StunTime>,
    requires_grounded: Has<UnStunOnlyAllowedWhenGrounded>,
    has_dead: Has<Dead>,
}

#[derive(SystemParam)]
pub struct StunSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    stunned_q: Query<'w, 's, StunnedAtQueryData, (With<Stunned>, Without<Dead>)>,
    grounded_sp: GroundedSystemParam<'w, 's>,
}

impl StunSystemParam<'_, '_> {
    fn unstun_expired(&mut self) {
        for stunned in self.stunned_q.iter() {
            let entity = stunned.entity;
            if stunned.has_dead {
                panic!("Stunned entity {entity} has Dead component");
            }
            let block = if stunned.requires_grounded {
                if let Some(grounded) = self.grounded_sp.is_grounded(stunned.entity) {
                    grounded
                } else {
                    warn!("UnStunOnlyAllowedWhenGrounded on entity {entity} that isn't Groundable");
                    false
                }
            } else {
                false
            };
            let Some(stun_time) = stunned.stun_time else {
                continue;
            };
            if block || stunned.stunned_at.0 + stun_time.0 > self.time.elapsed_secs_wrapped() {
                continue;
            }
            debug!("unstunning entity: {}", stunned.entity);
            let mut entity_cmds = self.commands.entity(stunned.entity);
            entity_cmds.remove::<Stunned>().trigger(OnUnStunned);
        }
    }
    pub fn stun(&mut self, entity: Entity) {
        debug!("stunning entity: {entity}");
        self.commands
            .entity(entity)
            .insert((Stunned, StunnedAt(self.time.elapsed_secs())))
            .trigger(OnStunned);
    }
    pub fn stun_with_time(&mut self, entity: Entity, stun_duration: f32) {
        debug!("stunning entity: {entity}");
        self.commands
            .entity(entity)
            .insert((
                Stunned,
                StunnedAt(self.time.elapsed_secs()),
                StunTime(stun_duration),
            ))
            .trigger(OnStunned);
    }
    pub fn is_stunned(&self, entity: Entity) -> bool {
        self.stunned_q.get(entity).is_ok()
    }
}

fn unstun_expired(mut stun_system_param: StunSystemParam) {
    stun_system_param.unstun_expired();
}

// TODO: see todo below - just manually calling triggers in functions
// fn on_add_stunned(trigger: Trigger<OnAdd, Stunned>, mut commands: Commands) {
//     debug!("Trigger<OnAdd, Stunned> stunning: {}", trigger.target());
//     commands.entity(trigger.target()).trigger(OnStunned);
// }

// TODO: bevy bug? despawn causes this trigger buffer thing to panic
// fn on_remove_stunned(trigger: Trigger<OnRemove, Stunned>, mut commands: Commands) {
//     let entity = trigger.target();
//     let Ok(mut entity_cmds) = commands.get_entity(entity) else {
//         return;
//     };
//     debug!("Trigger<OnRemove, Stunned> unstunning: {entity}");
//     entity_cmds.trigger(OnUnStunned);
// }

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    // app.add_observer(on_add_stunned);
    // app.add_observer(on_remove_stunned);
    app.add_systems(PostUpdate, unstun_expired);
}
