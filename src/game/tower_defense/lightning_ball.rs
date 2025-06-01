use crate::game::rng::global::GlobalRng;
use crate::game::rng::sphere::RandomSpherePoint;
use avian3d::prelude::{Collider, RigidBody};
use bevy::color::palettes::css::SKY_BLUE;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use itertools::Itertools;
use rand::Rng;
use smart_default::SmartDefault;
use std::f32::consts::{PI, TAU};
use std::ops::RangeInclusive;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(PointLight)]
#[require(Transform)]
#[require(LightningBallConfig)]
pub struct LightningBall;

#[auto_register_type]
#[derive(Component, Debug, SmartDefault, Clone, Reflect)]
#[reflect(Component)]
pub struct LightningBallConfig {
    #[default(DEFAULT_LIGHTNING_BALL_SPARK_RADIUS_MIN..=DEFAULT_LIGHTNING_BALL_SPARK_RADIUS_MAX)]
    pub spark_radius_range: RangeInclusive<f32>,
    #[default(DEFAULT_LIGHTNING_BALL_SPARK_COUNT)]
    pub spark_count: usize,
    #[default(DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_COUNT)]
    pub spark_segment_count: usize,
    #[default(DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_LEN_PERC)]
    pub spark_segment_len_perc: f32,
    #[default(DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_MAX_ANGLE_DEG)]
    pub spark_segment_max_angle_deg: f32,
}

pub const DEFAULT_LIGHTNING_BALL_RADIUS: f32 = 0.5;
pub const DEFAULT_LIGHTNING_BALL_SPARK_RADIUS_MIN: f32 = 1.0;
pub const DEFAULT_LIGHTNING_BALL_SPARK_RADIUS_MAX: f32 = 1.1;
pub const DEFAULT_LIGHTNING_BALL_SPARK_COUNT: usize = 10;
pub const DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_COUNT: usize = 3;
pub const DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_LEN_PERC: f32 = 0.25;
pub const DEFAULT_LIGHTNING_BALL_SPARK_SEGMENT_MAX_ANGLE_DEG: f32 = 45.0;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct LightningBallMeshCache(Handle<Mesh>);

impl FromWorld for LightningBallMeshCache {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self(meshes.add(Sphere::new(DEFAULT_LIGHTNING_BALL_RADIUS)))
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
        Collider::sphere(DEFAULT_LIGHTNING_BALL_RADIUS),
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
    pub lightning_ball_config: Mut<'static, LightningBallConfig>,
}

fn animate(
    mut gizmos: Gizmos,
    mut rng: GlobalRng,
    lightning_balls_q: Query<LightningBallQueryData, With<LightningBall>>,
) {
    for lb in lightning_balls_q.iter() {
        // Prevent crash during inspector editing and resulting in empty range
        if lb.lightning_ball_config.spark_radius_range.is_empty() {
            continue;
        }
        let scale = lb.transform.scale.length();
        let scaled_radius_min = lb.lightning_ball_config.spark_radius_range.start() * scale;
        let scaled_radius_max = lb.lightning_ball_config.spark_radius_range.end() * scale;
        let scaled_target_radius = (scaled_radius_min + scaled_radius_max) / 2.0;
        let total_spark_segment_length =
            TAU * lb.lightning_ball_config.spark_segment_len_perc * scale;
        let spark_segment_length =
            total_spark_segment_length / lb.lightning_ball_config.spark_segment_count as f32;
        let center = lb.global_transform.translation();

        for _ in 0..lb.lightning_ball_config.spark_count {
            // Pick a random starting point on the sphere:
            let transform_point = (*rng.rng()).random_sphere_point(scaled_target_radius);

            // Build a tangent‐space basis at transform_point:
            let normal = transform_point.normalize();
            // Pick any vector not parallel to `normal`:
            let helper = if normal.x.abs() < 0.9 {
                Vec3::X
            } else {
                Vec3::Y
            };
            // Gram–Schmidt to get a unit tangent:
            let tangent = (helper - normal * normal.dot(helper)).normalize();
            let bitangent = normal.cross(tangent);

            // Choose a truly random initial direction in that tangent plane:
            let phi = rng.rng().random_range(0.0f32..TAU);
            let mut direction = tangent * phi.cos() + bitangent * phi.sin();
            // Note: `direction` is a unit‐length vector, tangent to the sphere at transform_point.

            // Build the list of points (in *local* sphere coordinates):
            let mut points: Vec<Vec3> =
                Vec::with_capacity(lb.lightning_ball_config.spark_segment_count + 1);
            points.push(transform_point);
            let mut last = transform_point;

            let max_angle = lb
                .lightning_ball_config
                .spark_segment_max_angle_deg
                .to_radians();

            for _ in 0..lb.lightning_ball_config.spark_segment_count {
                // Rotate `direction` by a small random angle *around* the local normal:
                // (This “wiggles” the direction within the tangent plane.)
                let theta = rng.rng().random_range(-max_angle..=max_angle);
                let rot = Quat::from_axis_angle(normal, theta);
                direction = (rot * direction).normalize();

                // Take a small step “forward” along that tangent:
                let raw_offset = direction * spark_segment_length;
                let raw_next = last + raw_offset;

                // Pick a random radius in [min..max] for this new point:
                let rand_radius = rng
                    .rng()
                    .random_range(scaled_radius_min..=scaled_radius_max);

                // Project raw_next onto the sphere of that random radius:
                let next_on_sphere = raw_next.normalize() * rand_radius;
                points.push(next_on_sphere);
                last = next_on_sphere;
            }

            // Translate each point into world space and draw the polyline:
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
