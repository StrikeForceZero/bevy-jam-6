use crate::game::behaviors::stun::StunSystemParam;
use crate::game::effects::lightning_ball::LightningBallZappedBy;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ZapStuns;

#[auto_register_type]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(ZapStuns)]
pub struct ZapStunTime(pub f32);

fn refresh_zapped_by(
    zapped_by: Query<(Entity, &ZapStunTime, Ref<LightningBallZappedBy>)>,
    mut stun_sp: StunSystemParam,
) {
    struct ZappedByEntry {
        stun_time: f32,
    }
    let mut stunned = EntityHashMap::<ZappedByEntry>::default();
    for (entity, stun_time, zapped_by) in zapped_by.iter() {
        if zapped_by.is_added() {
            stunned.insert(
                entity,
                ZappedByEntry {
                    stun_time: stun_time.0,
                },
            );
        }
    }
    for (entity, entry) in stunned.into_iter() {
        stun_sp.stun_with_time(entity, entry.stun_time);
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PreUpdate, refresh_zapped_by);
}
