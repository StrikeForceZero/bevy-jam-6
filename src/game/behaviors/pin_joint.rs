use avian3d::prelude::JointDisabled;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PinJoints(EntityHashSet);

impl PinJoints {
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

#[auto_register_type]
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PinJoint(EntityHashMap<bool>);

impl PinJoint {
    pub fn is_enabled(&self) -> bool {
        self.0.iter().all(|(_, v)| *v)
    }
    pub fn new(a: Entity, b: Entity) -> Self {
        Self(EntityHashMap::from_iter([(a, true), (b, true)]))
    }
}

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Copy, Clone, Reflect)]
pub struct OnAddPinJoint(pub Entity);

fn on_add_pin_joint(
    trigger: Trigger<OnAddPinJoint>,
    mut pins_q: Query<Mut<PinJoints>, With<PinJoints>>,
) {
    let Ok(mut pins) = pins_q.get_mut(trigger.target()) else {
        return;
    };
    pins.0.insert(trigger.0);
}

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Copy, Clone, Reflect)]
pub struct DisablePinJoints;

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Debug, Copy, Clone, Reflect)]
pub struct EnablePinJoints;

fn enable_disable(
    pins_q: &Query<Ref<PinJoints>, With<PinJoints>>,
    pin_joints_q: &mut Query<Mut<PinJoint>, With<PinJoint>>,
    entity: Entity,
    enable: bool,
) {
    let Ok(pin_joints) = pins_q.get(entity) else {
        return;
    };
    for &pin_joint in pin_joints.iter() {
        let Ok(mut pin_joint) = pin_joints_q.get_mut(pin_joint) else {
            continue;
        };
        pin_joint.0.insert(entity, enable);
    }
}

fn on_enable_pin_joint(
    trigger: Trigger<EnablePinJoints>,
    pins_q: Query<Ref<PinJoints>, With<PinJoints>>,
    mut pin_joints_q: Query<Mut<PinJoint>, With<PinJoint>>,
) {
    let entity = trigger.target();
    enable_disable(&pins_q, &mut pin_joints_q, entity, true);
}

fn on_disable_pin_joint(
    trigger: Trigger<DisablePinJoints>,
    pins_q: Query<Ref<PinJoints>, With<PinJoints>>,
    mut pin_joints_q: Query<Mut<PinJoint>, With<PinJoint>>,
) {
    let entity = trigger.target();
    enable_disable(&pins_q, &mut pin_joints_q, entity, false);
}

fn update(mut commands: Commands, pin_joints_q: Query<(Entity, Ref<PinJoint>), Changed<PinJoint>>) {
    for (entity, pin_joint) in pin_joints_q.iter() {
        if !pin_joint.is_changed() {
            continue;
        }
        if pin_joint.is_enabled() {
            commands.entity(entity).remove::<JointDisabled>();
        } else {
            commands.entity(entity).insert(JointDisabled);
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_add_pin_joint);
    app.add_observer(on_enable_pin_joint);
    app.add_observer(on_disable_pin_joint);
    app.add_systems(FixedPreUpdate, update);
}
