use crate::util::trait_extension::MeshExt;
use crate::GameState;
use anyhow::{Result, Context};
use bevy::prelude::*;
use bevy_mod_sysfail::macros::*;
use bevy_rapier3d::prelude::*;

pub struct PhysicsPlugin;

/// Sets up the [`RapierPhysicsPlugin`] and [`RapierConfiguration`].
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
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
        if name.to_lowercase().contains("[collider]") || name.to_lowercase().contains("suzanne") {
            for (collider_entity, collider_mesh) in
                Mesh::search_in_children(entity, &children, &meshes, &mesh_handles)
            {
                let rapier_collider =
                    Collider::from_bevy_mesh(collider_mesh, &ComputedColliderShape::TriMesh)
                        .context("Failed to create collider from mesh")?;

                if let Some(mut entity_commands) = commands.get_entity(collider_entity) {
                    entity_commands.insert(rapier_collider);
                }
            }
        }
    }
    Ok(())
}
