use bevy::{asset::Assets, color::{palettes::css::WHITE, Color}, log::info, pbr::{MeshMaterial3d, PbrBundle, StandardMaterial}, prelude::{Commands, Component, Mesh, Mesh3d, ResMut, Transform}, utils::default};
use transvoxel::{prelude::Block, transition_sides};

use crate::utils::{format_value_f32, CHUNK_SIZE_F32, CHUNK_SIZE_I32};
use super::bevy_mesh::{mesh_for_model, Model};

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct Chunk;

pub fn build_chunk_mesh(cx: i32, cy: i32, cz: i32) -> Mesh {
    let block: Block<f32> = Block::from(
        [
            cx as f32 * CHUNK_SIZE_F32,
            cy as f32 * CHUNK_SIZE_F32,
            cz as f32 * CHUNK_SIZE_F32,
        ],
        CHUNK_SIZE_F32,
        CHUNK_SIZE_I32 as usize,
    );
    let transition_sides = transition_sides::no_side();
    mesh_for_model(&Model::Noise, false, &block, &transition_sides)
}

pub fn create_voxel_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = 4;
    let length = radius;
    let start_x = -radius;
    let start_z = -radius;

    for i in start_x..length {
        for j in start_z..length {
            let x = i;
            let z = j;
            info!(
                "Building Voxel Mesh @[{}, {}, {}]",
                format_value_f32(x as f32, None, true),
                format_value_f32(0.0, None, true),
                format_value_f32(z as f32, None, true)
            );
            let bevy_mesh = build_chunk_mesh(x, 0, z);
            // This object does not alter the transform as the transvoxel mesh using this information to sample the noise fields.
            
            commands.spawn((
                Mesh3d(meshes.add(bevy_mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WHITE.into(),
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
        }
    }
}