#![allow(unreachable_code)]

mod config;

use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

use crate::game::{constants::METERS_PER_UNIT, despawn::DespawnDelayed, health::Dead};

use super::{
    health::{AdjustHp, Health, MaxHealth},
    pause_controller::PausableSystems,
    snapshot::Snapshot,
};

use config::*;

#[auto_name]
#[auto_register_type]
#[derive(Component, Reflect, Default)]
#[require(Transform,Snapshot<GlobalTransform>)]
pub struct Spark;

#[auto_register_type]
#[derive(Component, Reflect, Default)]
#[require(Transform, Pickable)]
pub struct SparkTarget;

/// Spark -> Zapping -> SparkTarget
/// Inserts ChildOf
#[auto_register_type]
#[derive(Component, Reflect)]
#[require(Spark = enforce_exists!(Spark))]
#[relationship(relationship_target=ZappedBy)]
pub struct Zapping(pub Entity);

/// SparkTarget -> ZappedBy -> Spark
#[auto_register_type]
#[derive(Component, Reflect)]
#[require(SparkTarget = enforce_exists!(SparkTarget))]
#[relationship_target(relationship=Zapping)]
pub struct ZappedBy(Vec<Entity>);

#[auto_plugin(app=app)]
pub fn plugin(app: &mut App) {
    app.add_plugins(config::plugin);

    app.add_observer(SparkTarget::handle_inserted)
        .add_observer(Zapping::handle_inserted)
        .add_observer(Zapping::handle_removed)
        .add_observer(ZappedBy::handle_inserted)
        .add_observer(Spark::handle_inserted);

    app.add_systems(
        Update,
        (spark::decay_health, spark::deal_dot).in_set(PausableSystems),
    );

    app.add_systems(
        PostUpdate,
        spark::apply_distance_cost
            .in_set(PausableSystems)
            .after(TransformSystem::TransformPropagate),
    );
}

impl SparkTarget {
    fn handle_inserted(tr: Trigger<OnInsert, Self>, mut commands: Commands) {
        fn handle_clicked(
            tr: Trigger<Pointer<Click>>,
            mut commands: Commands,
            sparks: Query<(Entity, &GlobalTransform), With<Spark>>,
            targets: Query<&GlobalTransform, With<SparkTarget>>,
            cfg: Res<SparkConfig>,
        ) {
            let tl_target = targets
                .get(tr.target())
                .expect("cause of picking")
                .translation();

            for (spark, tf_spark) in sparks {
                let dist = (tf_spark.translation() - tl_target).length() * METERS_PER_UNIT;
                if dist > cfg.max_distance_jump_m {
                    continue;
                }

                commands
                    .entity(spark)
                    .remove::<Zapping>()
                    .insert(Zapping(tr.target()));
            }
        }

        commands.entity(tr.target()).observe(handle_clicked);
    }
}

impl Zapping {
    /// Updates parenting
    fn handle_inserted(
        tr: Trigger<OnInsert, Self>,
        comp: Query<&Self, Added<Self>>,
        mut commands: Commands,
    ) {
        let comp = comp.get(tr.target()).expect("OnInsert broken");
        commands
            .entity(tr.target())
            .insert((ChildOf(comp.0), Transform::default())); // TODO better relative positioning
    }

    fn handle_removed(
        tr: Trigger<OnRemove, Self>,
        mut sparks: Query<(&mut Transform, &GlobalTransform), With<Spark>>,
        mut commands: Commands,
    ) {
        let (mut tf, gl_tf) = sparks.get_mut(tr.target()).expect("require");
        tf.translation = gl_tf.translation();

        commands.entity(tr.target()).try_remove::<ChildOf>();
    }
}

impl ZappedBy {
    fn handle_inserted(tr: Trigger<OnInsert, Self>, mut commands: Commands) {
        fn handle_death(tr: Trigger<OnInsert, Dead>, mut commands: Commands) {
            commands.entity(tr.target()).remove::<ZappedBy>();
        }

        commands.entity(tr.target()).observe(handle_death);
    }
}

impl Spark {
    fn handle_inserted(tr: Trigger<OnInsert, Self>, mut commands: Commands, cfg: Res<SparkConfig>) {
        fn handle_death(tr: Trigger<OnInsert, Dead>, mut commands: Commands) {
            commands.entity(tr.target()).trigger(DespawnDelayed);
        }

        commands
            .entity(tr.target())
            .insert((Health(cfg.start_charge), MaxHealth(cfg.max_charge)))
            .observe(handle_death);
    }
}

#[allow(clippy::module_inception)]
mod spark {
    use super::*;

    pub fn decay_health(
        sparks: Query<Entity, With<Spark>>,
        time: Res<Time>,
        mut adjust_hp_event: EventWriter<AdjustHp>,
        cfg: Res<SparkConfig>,
    ) {
        let damage_amount = time.delta_secs() * cfg.decay_per_second;

        adjust_hp_event.write_batch(
            sparks
                .iter()
                .map(|spark| AdjustHp::new(spark, -damage_amount)),
        );
    }

    pub fn deal_dot(
        targets: Query<Entity, (With<ZappedBy>, With<Health>, Without<Dead>)>,
        time: Res<Time>,
        mut adjust_hp_event: EventWriter<AdjustHp>,
        cfg: Res<SparkConfig>,
    ) {
        let damage_amount = time.delta_secs() * cfg.damage_dealt_per_second;

        adjust_hp_event.write_batch(
            targets
                .iter()
                .map(|target| AdjustHp::new(target, -damage_amount)),
        );
    }

    pub fn apply_distance_cost(
        mut sparks: Query<
            (Entity, &GlobalTransform, &mut Snapshot<GlobalTransform>),
            (With<Spark>, Changed<GlobalTransform>),
        >,
        mut adjust_hp_event: EventWriter<AdjustHp>,
        cfg: Res<SparkConfig>,
    ) {
        for (spark, gt_new, mut gt_snap) in sparks.iter_mut() {
            let Some(gt_old) = gt_snap.replace(*gt_new) else {
                continue;
            };

            let dist = (gt_new.translation() - gt_old.translation()).length() * METERS_PER_UNIT;

            adjust_hp_event.write(AdjustHp::new(spark, -dist * cfg.cost_per_m));
        }
    }
}
