use crate::game::rng::global::GlobalRng;
use crate::game::rng::sphere::RandomSpherePoint;
use avian3d::prelude::{Collider, RigidBody};
use bevy::color;
use bevy::color::palettes::css::SKY_BLUE;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use itertools::Itertools;
use rand::Rng;
use std::f32::consts::PI;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(PointLight)]
#[require(Transform)]
pub struct LightningBall;

pub const LIGHTNING_BALL_RADIUS: f32 = 10.0;
pub const LIGHTNING_BALL_SPARK_COUNT: usize = 10;
pub const LIGHTNING_BALL_SPARK_SEGMENT_COUNT: usize = 2;
pub const LIGHTNING_BALL_SPARK_SEGMENT_LEN: f32 =
    2.0 * PI * LIGHTNING_BALL_RADIUS / 4.0 / LIGHTNING_BALL_SPARK_SEGMENT_COUNT as f32;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LightningBallMeshCache(Handle<Mesh>);

impl FromWorld for LightningBallMeshCache {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self(meshes.add(Sphere::new(LIGHTNING_BALL_RADIUS)))
    }
}

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LightningBallMeshMaterialCache(Handle<StandardMaterial>);

impl FromWorld for LightningBallMeshMaterialCache {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self(materials.add(StandardMaterial {
            emissive: SKY_BLUE.into(),
            ..Default::default()
        }))
    }
}

fn on_lightning_ball_added(
    trigger: Trigger<OnAdd, LightningBall>,
    mut commands: Commands,
    material_cache: Res<LightningBallMeshMaterialCache>,
    mesh_cache: Res<LightningBallMeshCache>,
) {
    commands.entity(trigger.target()).insert((
        Mesh3d(mesh_cache.0.clone()),
        MeshMaterial3d(material_cache.0.clone()),
        PointLight {
            color: SKY_BLUE.into(),
            intensity: 99999999.0,
            range: 1000.0,
            radius: 999.0,
            shadows_enabled: true,
            ..Default::default()
        },
        RigidBody::Kinematic,
        Collider::sphere(LIGHTNING_BALL_RADIUS),
    ));
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct LightningBallQueryData {
    pub entity: Entity,
    pub lightning_ball: Ref<'static, LightningBall>,
    pub transform: Mut<'static, Transform>,
    pub global_transform: Ref<'static, GlobalTransform>,
    pub point_light: Mut<'static, PointLight>,
}

const SPARK_OFFSET: f32 = LIGHTNING_BALL_RADIUS * 1.05 - LIGHTNING_BALL_RADIUS;
fn animate(
    mut gizmos: Gizmos,
    mut rng: GlobalRng,
    lightning_balls_q: Query<LightningBallQueryData, With<LightningBall>>,
) {
    for lb in lightning_balls_q.iter() {
        for _ in 0..LIGHTNING_BALL_SPARK_COUNT {
            let direction = (*rng.rng()).random_range(-PI..=PI);
            let direction = Vec3::new(direction.cos(), direction.sin(), 0.0);
            let transform_point =
                (*rng.rng()).random_sphere_point(LIGHTNING_BALL_RADIUS + SPARK_OFFSET);
            let points =
                (0..LIGHTNING_BALL_SPARK_SEGMENT_COUNT).fold(vec![Vec3::ZERO], |mut points, _| {
                    let last_point = points.last().expect("unreachable");
                    let next_point_angle = rng.rng().random_range(-45..=45) as f32;
                    let next_point_height = rng.rng().random_range(-SPARK_OFFSET..=SPARK_OFFSET);
                    let next_point = Quat::from_rotation_z(next_point_angle)
                        * (last_point + direction * LIGHTNING_BALL_SPARK_SEGMENT_LEN)
                        + Vec3::Y * next_point_height;
                    points.push(next_point);
                    points
                });
            let points = points
                .into_iter()
                .map(|p| {
                    Transform::from_translation(transform_point + lb.global_transform.translation())
                        .transform_point(p)
                })
                .collect::<Vec<_>>();
            for (&a, &b) in points.iter().tuple_windows() {
                gizmos.line_gradient(a, b, Color::WHITE, Color::from(SKY_BLUE));
            }
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_lightning_ball_added);
    app.add_systems(Update, animate);
}
