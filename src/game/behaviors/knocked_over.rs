use crate::game::utils::quat::get_pitch_and_roll;
use bevy::ecs::query::{QueryData, QueryEntityError};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct KnockedOver;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct KnockedOverAngle(pub f32);

#[derive(QueryData)]
#[query_data(derive(Debug))]
pub struct KnockedOverQueryData {
    pub entity: Entity,
    pub knocked_over: Option<Ref<'static, KnockedOver>>,
    pub knocked_over_angle: &'static KnockedOverAngle,
    pub transform: &'static Transform,
}

impl KnockedOverQueryDataItem<'_> {
    /// only use after [`update_knocked_over`] has ran
    pub fn has_knocked_over(&self) -> bool {
        self.knocked_over.is_some()
    }
    /// calculates if is knocked over by the given [`KnockedOverAngle`]
    pub fn calc_is_knocked_over(&self) -> bool {
        self.current_pitch_angle().to_degrees() >= self.knocked_over_angle.0.to_degrees()
    }
    pub fn current_pitch_angle(&self) -> f32 {
        let rot = self.transform.rotation;
        let (pitch, _roll) = get_pitch_and_roll(rot);
        pitch.abs()
    }
}

#[derive(SystemParam)]
pub struct KnockedOverSystemParams<'w, 's> {
    pub knocked_over_q: Query<'w, 's, KnockedOverQueryData, With<KnockedOverAngle>>,
}

impl KnockedOverSystemParams<'_, '_> {
    /// only use after [`update_knocked_over`] has ran
    pub fn has_knocked_over(&self, entity: Entity) -> Result<bool, QueryEntityError> {
        self.knocked_over_q
            .get(entity)
            .map(|knocked_over_q| knocked_over_q.has_knocked_over())
    }
    /// calculates if is knocked over by the given [`KnockedOverAngle`]
    pub fn calc_is_knocked_over(&self, entity: Entity) -> Result<bool, QueryEntityError> {
        self.knocked_over_q
            .get(entity)
            .map(|item| item.calc_is_knocked_over())
    }
}

fn update_knocked_over(mut commands: Commands, knocked_over_sp: KnockedOverSystemParams) {
    for item in knocked_over_sp.knocked_over_q.iter() {
        if item.calc_is_knocked_over() {
            if !item.has_knocked_over() {
                commands.entity(item.entity).insert(KnockedOver);
            }
        } else {
            commands.entity(item.entity).remove::<KnockedOver>();
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, update_knocked_over);
}
