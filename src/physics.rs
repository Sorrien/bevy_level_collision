use crate::util::trait_extension::MeshExt;
use crate::GameState;
use anyhow::{Context, Result};
use bevy::{prelude::*, render::mesh::{VertexAttributeValues, Indices}};
use bevy_mod_sysfail::macros::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::SharedShape, na::Point3};

pub struct PhysicsPlugin;

/// Sets up the [`RapierPhysicsPlugin`] and [`RapierConfiguration`].
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            timestep_mode: TimestepMode::Variable {
                max_dt: 1.0 / 20.0,
                time_scale: 1.0,
                substeps: 4,
            },
            ..default()
        })
        .add_systems(Update, read_colliders.run_if(in_state(GameState::Playing)));
    }
}

#[sysfail(log(level = "error"))]
pub(crate) fn read_colliders(
    mut commands: Commands,
    added_name: Query<(Entity, &Name), Added<Name>>,
    children: Query<&Children>,
    meshes: Res<Assets<Mesh>>,
    mesh_handles: Query<&Handle<Mesh>>,
) -> Result<()> {
    #[cfg(feature = "tracing")]
    let _span = info_span!("read_colliders").entered();
    for (entity, name) in &added_name {
        if name.to_lowercase().contains("[collider]") {
            for (collider_entity, collider_mesh) in
                Mesh::search_in_children(entity, &children, &meshes, &mesh_handles)
            {
                println!("{:?}", name);
                let rapier_collider =
                    from_bevy_mesh(collider_mesh, &ComputedColliderShape::TriMesh)
                        .context("Failed to create collider from mesh")?;

                if let Some(mut entity_commands) = commands.get_entity(collider_entity) {
                    entity_commands.insert(rapier_collider);
                }
            }
        }
    }
    Ok(())
}

/// Initializes a collider with a Bevy Mesh.
    ///
    /// Returns `None` if the index buffer or vertex buffer of the mesh are in an incompatible format.
    pub fn from_bevy_mesh(mesh: &Mesh, collider_shape: &ComputedColliderShape) -> Option<Collider> {
        let Some((vtx, idx)) = extract_mesh_vertices_indices(mesh) else { return None; };
        match collider_shape {
            ComputedColliderShape::TriMesh => Some(
                SharedShape::trimesh_with_flags(vtx, idx, TriMeshFlags::MERGE_DUPLICATE_VERTICES)
                    .into(),
            ),
            ComputedColliderShape::ConvexHull => {
                SharedShape::convex_hull(&vtx).map(|shape| shape.into())
            }
            ComputedColliderShape::ConvexDecomposition(params) => {
                Some(SharedShape::convex_decomposition_with_params(&vtx, &idx, params).into())
            }
        }
    }

    fn extract_mesh_vertices_indices(mesh: &Mesh) -> Option<(Vec<Point3<Real>>, Vec<[u32; 3]>)> {
    use bevy_rapier3d::na::point;
        let vertices = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;
        let indices = mesh.indices()?;

        println!("{:?}", vertices);
    
        let vtx: Vec<_> = match vertices {
            VertexAttributeValues::Float32(vtx) => Some(
                vtx.chunks(3)
                    .map(|v| point![v[0] as Real, v[1] as Real, v[2] as Real])
                    .collect(),
            ),
            VertexAttributeValues::Float32x3(vtx) => Some(
                vtx.iter()
                    .map(|v| point![v[0] as Real, v[1] as Real, v[2] as Real])
                    .collect(),
            ),
            _ => None,
        }?;
    
        let idx = match indices {
            Indices::U16(idx) => idx
                .chunks_exact(3)
                .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
                .collect(),
            Indices::U32(idx) => idx.chunks_exact(3).map(|i| [i[0], i[1], i[2]]).collect(),
        };

        //println!("{:?} {:?}", vtx, idx);
        println!("{:?}", vtx);

        Some((vtx, idx))
    }