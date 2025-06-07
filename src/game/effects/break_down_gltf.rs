use crate::game::screens::loading::all_assets_loaded;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::gltf::GltfMesh;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy_auto_plugin::auto_plugin::*;

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct BreakableGltfs(HashSet<Handle<Gltf>>);

impl BreakableGltfs {
    pub fn add(&mut self, gltf: Handle<Gltf>) {
        self.0.insert(gltf);
    }
}

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Debug, Default, Clone, Reflect)]
#[reflect(Resource)]
pub struct UnskinnedMeshMap(HashMap<Handle<Mesh>, Handle<Mesh>>);

/// Removes attributes from Mesh that would make the prepass pipeline expect SkinnedMesh components
fn strip_skinned_attributes(mesh: &mut Mesh) {
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_INDEX);
    mesh.remove_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT);
}

pub fn extract_unskinned_gltf_mesh_map(
    gltf: &Gltf,
    gltf_meshes: &Assets<GltfMesh>,
    meshes: &mut Assets<Mesh>,
) -> HashMap<Handle<Mesh>, Handle<Mesh>> {
    let mut result: HashMap<Handle<Mesh>, Handle<Mesh>> = HashMap::new();
    for mesh_handle in &gltf.meshes {
        let Some(gltf_mesh) = gltf_meshes.get(mesh_handle) else {
            // TODO: return Err("GltfMesh not loaded in asset server")
            continue;
        };
        for primitive in &gltf_mesh.primitives {
            let Some(mesh) = meshes.get(&primitive.mesh) else {
                // TODO: return Err("Mesh not loaded in asset server")
                continue;
            };
            let mut unskinned_mesh = mesh.clone();
            strip_skinned_attributes(&mut unskinned_mesh);
            assert!(
                result
                    .insert(primitive.mesh.clone(), meshes.add(unskinned_mesh),)
                    .is_none()
            );
        }
    }

    result
}

fn init_unskinned_mesh_map(
    mut done: Local<bool>,
    breakable_gltfs: Res<BreakableGltfs>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut unskinned_mesh_map: ResMut<UnskinnedMeshMap>,
) {
    // only run once
    if *done {
        return;
    }
    *done = true;

    for gltf_handle in breakable_gltfs.0.iter() {
        let gltf = gltfs
            .get(gltf_handle.id())
            .expect("expected gltf to be preloaded");
        for (key, value) in extract_unskinned_gltf_mesh_map(gltf, &gltf_meshes, &mut meshes) {
            let result = unskinned_mesh_map.0.insert(key, value);
            assert!(result.is_none(), "Already unskinned mesh");
        }
    }
}

#[derive(QueryData)]
struct MeshMaterialQueryData {
    mesh_3d: &'static Mesh3d,
    mesh_material_3d: &'static MeshMaterial3d<StandardMaterial>,
    global_transform: &'static GlobalTransform,
}

#[derive(SystemParam)]
pub struct BreakGltfSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    unskinned_mesh_map: Res<'w, UnskinnedMeshMap>,
    child_of_q: Query<'w, 's, &'static ChildOf>,
    children_q: Query<'w, 's, &'static Children>,
    mesh_material_q: Query<'w, 's, MeshMaterialQueryData>,
}

impl BreakGltfSystemParam<'_, '_> {
    pub fn break_gltf(&mut self, entity: Entity, despawn_original: bool) -> Vec<Entity> {
        let child_of_opt = self.child_of_q.get(entity).ok();

        let mut parts = Vec::new();

        for child in self.children_q.iter_descendants(entity) {
            if despawn_original {
                self.commands.entity(child).try_despawn();
            }

            let Ok(MeshMaterialQueryDataItem {
                mesh_3d,
                mesh_material_3d,
                global_transform,
            }) = self.mesh_material_q.get(child)
            else {
                continue;
            };

            let unskinned_mesh_handle = self
                .unskinned_mesh_map
                .0
                .get(&mesh_3d.0)
                .unwrap_or_else(|| panic!("missing unskinned mesh for {child}"))
                .clone();
            let part = self
                .commands
                .spawn((
                    Mesh3d(unskinned_mesh_handle),
                    mesh_material_3d.clone(),
                    Transform::from_matrix(global_transform.compute_matrix()),
                ))
                .id();

            parts.push(part);

            // TODO: probably use the spawn helper?
            if let Some(&ChildOf(parent)) = child_of_opt {
                self.commands.entity(parent).add_child(part);
            }
        }

        if despawn_original {
            self.commands.entity(entity).try_despawn();
        }

        parts
    }
}

#[auto_plugin(app=app)]
pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, init_unskinned_mesh_map.run_if(all_assets_loaded));
}
