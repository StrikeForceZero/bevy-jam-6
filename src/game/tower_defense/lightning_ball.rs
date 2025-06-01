use crate::game::rng::global::GlobalRng;
use crate::game::rng::sphere::RandomSpherePoint;
use avian3d::prelude::{Collider, RigidBody};
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
pub const LIGHTNING_BALL_SPARK_SEGMENT_COUNT: usize = 3;
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

const SPARK_OFFSET: f32 = LIGHTNING_BALL_RADIUS * 1.1 - LIGHTNING_BALL_RADIUS;
const SPARK_RADIUS: f32 = LIGHTNING_BALL_RADIUS + SPARK_OFFSET;

fn animate(
    mut gizmos: Gizmos,
    mut rng: GlobalRng,
    lightning_balls_q: Query<LightningBallQueryData, With<LightningBall>>,
) {
    for lb in lightning_balls_q.iter() {
        // Center of this lightning ball
        let center = lb.global_transform.translation();

        for _ in 0..LIGHTNING_BALL_SPARK_COUNT {
            // Pick a random starting point exactly on the sphere of radius SPARK_RADIUS:
            let transform_point = (*rng.rng()).random_sphere_point(SPARK_RADIUS);

            // Build the spark‐segment polyline, but each time project back onto the sphere:
            let mut points: Vec<Vec3> = Vec::with_capacity(LIGHTNING_BALL_SPARK_SEGMENT_COUNT + 1);
            points.push(transform_point);

            for _ in 0..LIGHTNING_BALL_SPARK_SEGMENT_COUNT {
                let Some(last) = points.last() else {
                    unreachable!()
                };

                // Compute some “raw” offset in 3D. For example, take a random small angular step
                //  around the Z‐axis, and a tiny wiggle in the Y‐direction:
                let angle_deg: f32 = rng.rng().random_range(-45.0..=45.0);
                let rot = Quat::from_rotation_z(angle_deg.to_degrees());

                // Move “forward” by rotating the last‐point around Z, then push it outwards:
                let raw_next =
                    rot * (last + Vec3::new(1.0, 0.0, 0.0) * LIGHTNING_BALL_SPARK_SEGMENT_LEN);
                let rand_height = rng.rng().random_range(-SPARK_OFFSET..=SPARK_OFFSET);

                // Project that “raw_next” back onto the sphere radius SPARK_RADIUS + rand_height
                let next_on_sphere = raw_next.normalize() * (SPARK_RADIUS + rand_height);
                points.push(next_on_sphere);
            }

            // Translate every point by the ball’s center and rotate it to the random point vector
            let world_pts: Vec<Vec3> = points
                .into_iter()
                .map(|p| Transform::from_translation(center).transform_point(p))
                .collect();

            for (&a, &b) in world_pts.iter().tuple_windows() {
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
