use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct TargetEnt {
    pub target_ent: Entity,
    pub within_distance: f32,
}

fn target_ent_sys(
    mut commands: Commands,
    time: Res<Time>,
    target_q: Query<(Entity, &TargetEnt)>,
    mut transform_q: Query<&mut Transform>,
) {
    // TODO parameterize/componetize speed
    let speed: f32 = 5. * time.delta_secs();
    for (self_ent, &target) in target_q.iter() {
        let target_ent = target.target_ent;
        // If target ent no longer exists, remove component
        let Ok(target_trans) = transform_q.get(target_ent).cloned() else {
            commands.entity(self_ent).remove::<TargetEnt>();
            return;
        };
        let mut self_trans = transform_q.get_mut(self_ent).unwrap();
        self_trans.look_at(target_trans.translation, Vec3::Y);
        let dist = self_trans.translation.distance(target_trans.translation);
        if dist > target.within_distance {
            let move_dist = speed.min(dist - target.within_distance);
            self_trans.translation = self_trans
                .translation
                .move_towards(target_trans.translation, move_dist);
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, target_ent_sys);
}
