// TODO: split bolt into component marker with behavior and effects

use crate::game::rng::global::GlobalRng;
use crate::game::rng::sphere::RandomSpherePoint;
use avian3d::prelude::{
    Collider, CollidingEntities, Collisions, Position, RigidBody, Rotation, Sensor,
};
use bevy::color::palettes::css::SKY_BLUE;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use itertools::Itertools;
use rand::Rng;
use smart_default::SmartDefault;
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Default, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(PointLight)]
#[require(Transform)]
#[require(LightningBallConfig)]
#[require(LightningBallSources)]
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
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[relationship_target(relationship = LightningBallSource, linked_spawn)]
struct LightningBallSources(Vec<Entity>);

#[auto_register_type]
#[auto_name]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[relationship(relationship_target = LightningBallSources)]
struct LightningBallSource(Entity);

#[auto_register_type]
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Transform)]
#[require(Collider)]
pub struct LightningBallConduit;

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
    let entity = trigger.target();
    commands.entity(entity).insert((
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
        children![(
            LightningBallSource(entity),
            Sensor,
            Collider::sphere(DEFAULT_LIGHTNING_BALL_RADIUS * 50.0),
            CollidingEntities::default(),
        )],
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
    lighting_ball_sources: Ref<'static, LightningBallSources>,
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct LightningBallSourceQueryData {
    pub entity: Entity,
    lightning_ball_source: Ref<'static, LightningBallSource>,
    pub transform: Mut<'static, Transform>,
    pub global_transform: Ref<'static, GlobalTransform>,
    pub colliding_entities: Ref<'static, CollidingEntities>,
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

fn animate_in_range(
    mut gizmos: Gizmos,
    mut rng: GlobalRng,
    lightning_balls_q: Query<
        LightningBallQueryData,
        (With<LightningBall>, Without<LightningBallSource>),
    >,
    lightning_balls_source_q: Query<
        LightningBallSourceQueryData,
        (With<LightningBallSource>, Without<LightningBall>),
    >,
    colliding_q: Query<(&Position, &Rotation), With<LightningBallConduit>>,
    collisions: Collisions,
) {
    for lb in lightning_balls_q.iter() {
        // Prevent crash during inspector editing and resulting in empty range
        if lb.lightning_ball_config.spark_radius_range.is_empty() {
            continue;
        }
        let scale = lb.transform.scale.length();
        let scaled_radius_max = lb.lightning_ball_config.spark_radius_range.end() * scale;
        let center = lb.global_transform.translation();

        for lightning_ball_source_entity in lb.lighting_ball_sources.iter() {
            let Ok(lb_source) = lightning_balls_source_q.get(lightning_ball_source_entity) else {
                continue;
            };
            for _ in 0..lb.lightning_ball_config.spark_count {
                for &colliding_entity in lb_source.colliding_entities.iter() {
                    let Ok((position, rotation)) = colliding_q.get(colliding_entity) else {
                        continue;
                    };

                    let Some(cp) = collisions.get(lb_source.entity, colliding_entity) else {
                        continue;
                    };

                    let Some(contact) = cp.find_deepest_contact() else {
                        continue;
                    };

                    let target_world_pos = if cp.collider1 == colliding_entity {
                        contact.global_point1(position, rotation)
                    } else if cp.collider2 == colliding_entity {
                        contact.global_point2(position, rotation)
                    } else {
                        unreachable!("bad collision");
                    };

                    // Direction + distance from ball center → target
                    let to_target = target_world_pos - center;
                    if to_target.length_squared() < f32::EPSILON {
                        continue;
                    }
                    let n = to_target.normalize(); // “pole” for our hemisphere
                    let full_distance = to_target.length();

                    // Sphere’s outer radius (so we launch exactly from the curved face):
                    let sphere_radius: f32 = scaled_radius_max;

                    // Build an orthonormal basis {perp1, perp2} ⟂ n:
                    let helper = if n.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
                    let perp1 = (helper - n * n.dot(helper)).normalize();
                    let perp2 = n.cross(perp1);

                    // Sample a random direction on the hemisphere whose “pole” is n:
                    let phi = rng.rng().random_range(0.0f32..TAU);
                    let cos_theta = rng.rng().random_range(0.0f32..=1.0f32);
                    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                    let hemisphere_dir = perp1 * (sin_theta * phi.cos())
                        + perp2 * (sin_theta * phi.sin())
                        + n * (cos_theta);

                    // Compute the random start point on that hemisphere of the sphere:
                    let start_point: Vec3 = center + hemisphere_dir * sphere_radius;

                    // TODO: space out bends to make it more organic
                    // Build a jagged bolt from start_point → target_world_pos:
                    //    interpolate from “start_point on the sphere surface” to the exact target,
                    //    adding a small sideways jitter at each interior segment.
                    let mut bolt_points: Vec<Vec3> =
                        Vec::with_capacity(lb.lightning_ball_config.spark_segment_count + 2);
                    bolt_points.push(start_point);

                    // Precompute straight vs lateral:
                    let segments = lb.lightning_ball_config.spark_segment_count as f32;
                    for i in 1..=lb.lightning_ball_config.spark_segment_count {
                        let t = (i as f32) / (segments + 1.0);
                        // Interpolate the distance from start_radius → full_distance:
                        let distance_along_line =
                            sphere_radius + (full_distance - sphere_radius) * t;
                        let ideal_point = center + n * distance_along_line;

                        // Sideways jitter: zero at t=0, zero at t=1, peaks at t=0.5
                        let max_offset = full_distance * 0.05 * (1.0 - (2.0 * (t - 0.5)).abs());
                        let mag = rng.rng().random_range(-max_offset..=max_offset);
                        let angle = rng.rng().random_range(0.0f32..TAU);
                        let sideways_offset =
                            perp1 * (angle.cos() * mag) + perp2 * (angle.sin() * mag);

                        let next_pt = ideal_point + sideways_offset;
                        bolt_points.push(next_pt);
                    }

                    // End at the collider’s transform:
                    bolt_points.push(target_world_pos);

                    // Draw
                    for (&a, &b) in bolt_points.iter().tuple_windows() {
                        gizmos.line_gradient(a, b, Color::WHITE, Color::from(SKY_BLUE));
                    }
                }
            }
        }
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_observer(on_lightning_ball_added);
    app.add_systems(Update, (animate, animate_in_range));
}
