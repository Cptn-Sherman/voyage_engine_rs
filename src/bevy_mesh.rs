/*!
This module contains mesh builders to produce [Bevy](https://bevyengine.org/) meshes.
*/

#[derive(Default)]
pub struct BevyMeshBuilder {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangle_indices: Vec<u32>,
    vertices: usize,
}

/// A simple bevy mesh builder that:
///  - only populates position/normal attributes
///  - only looks at density of the VoxelData
impl BevyMeshBuilder {
    /**
    Build a Bevy mesh, producing a triangle list mesh with positions and normals
    from our mesh, but UV coordinates all set to 0
    */
    pub fn build(self) -> Mesh {
        let mut bevy_mesh = Mesh::new(TriangleList, RenderAssetUsages::MAIN_WORLD);
        bevy_mesh.insert_indices(Indices::U32(self.triangle_indices));
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        return bevy_mesh;
    }
    /**
    Convert to a Bevy mesh lines list, with positions and normals
    from our mesh, but UV coordinates all set to 0.
    Lines shared between 2 triangles are repeated, for implementation simplicity.
    */
    pub fn build_wireframe(self) -> Mesh {
        let mut bevy_mesh = Mesh::new(LineList, RenderAssetUsages::MAIN_WORLD);
        let tris_count = self.triangle_indices.len() / 3;
        let indices = (0..tris_count)
            .map(|i| vec![3 * i, 3 * i + 1, 3 * i + 1, 3 * i + 2, 3 * i + 2, 3 * i])
            .flatten()
            .map(|j| self.triangle_indices[j] as u32)
            .collect();
        bevy_mesh.insert_indices(Indices::U32(indices));
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        return bevy_mesh;
    }
}

impl<V> MeshBuilder<V, f32> for BevyMeshBuilder
where
    V: VoxelData<Density = f32>,
{
    fn add_vertex_between(
        &mut self,
        point_a: GridPoint<V, f32>,
        point_b: GridPoint<V, f32>,
        interp_toward_b: f32,
    ) -> VertexIndex {
        let position = point_a
            .position
            .interp_toward(&point_b.position, interp_toward_b);
        let gradient_x =
            point_a.gradient.0 + interp_toward_b * (point_b.gradient.0 - point_a.gradient.0);
        let gradient_y =
            point_a.gradient.1 + interp_toward_b * (point_b.gradient.1 - point_a.gradient.1);
        let gradient_z =
            point_a.gradient.2 + interp_toward_b * (point_b.gradient.2 - point_a.gradient.2);
        let normal = f32::gradients_to_normal(gradient_x, gradient_y, gradient_z);
        self.positions.push([position.x, position.y, position.z]);
        self.normals.push(normal);
        let index = self.vertices;
        self.vertices += 1;
        return VertexIndex(index);
    }

    fn add_triangle(
        &mut self,
        vertex_1_index: VertexIndex,
        vertex_2_index: VertexIndex,
        vertex_3_index: VertexIndex,
    ) {
        self.triangle_indices.push(vertex_1_index.0 as u32);
        self.triangle_indices.push(vertex_2_index.0 as u32);
        self.triangle_indices.push(vertex_3_index.0 as u32);
    }
}

use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::render::mesh::Indices;
use bevy::render::mesh::Mesh as BevyMesh;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology::{LineList, TriangleList};
use transvoxel::mesh_builder::GridPoint;
use transvoxel::mesh_builder::MeshBuilder;
use transvoxel::mesh_builder::VertexIndex;
use transvoxel::shrink_if_needed;
use transvoxel::traits::Density;
use transvoxel::traits::VoxelData;
use transvoxel::transition_sides::*;
use transvoxel::voxel_source::Block;
use transvoxel::{
    extraction::extract,
    voxel_coordinates::{HighResolutionVoxelDelta, TransitionCellIndex},
    voxel_source::WorldMappingVoxelSource,
};

pub fn mesh_for_model(
    model: &Model,
    wireframe: bool,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let mut models_map = models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    field_model(field, wireframe, block, transition_sides)
}

pub fn inside_grid_points(
    model: &Model,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut models_map = models_map();
    let field = models_map.get_mut(model).unwrap().as_mut();
    inside_grid_points_for_field(field, block, transition_sides)
}

fn field_model(
    field: &mut dyn DataField<f32, f32>,
    wireframe: bool,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> BevyMesh {
    let source = WorldMappingVoxelSource {
        field: field,
        block: &block,
    };
    let builder = extract(
        source,
        &block,
        THRESHOLD,
        *transition_sides,
        BevyMeshBuilder::default(),
    );
    if wireframe {
        builder.build_wireframe()
    } else {
        builder.build()
    }
}

fn inside_grid_points_for_field(
    field: &mut dyn DataField<f32, f32>,
    block: &Block<f32>,
    transition_sides: &TransitionSides,
) -> Vec<(f32, f32, f32)> {
    let mut result = Vec::<(f32, f32, f32)>::new();
    // Regular points (some shrunk)
    for i in 0..=block.subdivisions {
        for j in 0..=block.subdivisions {
            for k in 0..=block.subdivisions {
                let unshrunk_pos = regular_position(block, i, j, k, &no_side());
                let final_pos = regular_position(block, i, j, k, transition_sides);
                let d = field.get_data(unshrunk_pos[0], unshrunk_pos[1], unshrunk_pos[2]);
                let inside = d >= THRESHOLD;
                if inside {
                    result.push((final_pos[0], final_pos[1], final_pos[2]));
                }
            }
        }
    }
    // Hig-res faces points
    for side in *transition_sides {
        for u in 0..=(block.subdivisions * 2) {
            for v in 0..=(block.subdivisions * 2) {
                let voxel_index = &TransitionCellIndex::from(side, 0, 0)
                    + &HighResolutionVoxelDelta::from(u as isize, v as isize, 0);
                let position_in_block = voxel_index.to_position_in_block(block);
                let pos = &(&position_in_block * block.dims.size) + &block.dims.base;
                let d = field.get_data(pos.x, pos.y, pos.z);
                let inside = d >= THRESHOLD;
                if inside {
                    result.push((pos.x, pos.y, pos.z));
                }
            }
        }
    }
    return result;
}

pub fn grid_lines(block: &Block<f32>, transition_sides: &TransitionSides) -> BevyMesh {
    let subs = block.subdivisions;
    let mut bevy_mesh = BevyMesh::new(bevy::render::render_resource::PrimitiveTopology::LineList, RenderAssetUsages::MAIN_WORLD);
    let mut positions = Vec::<[f32; 3]>::new();
    let mut indices = Vec::<u32>::new();
    for i in 0..=subs {
        for j in 0..=subs {
            // Z-line
            if subs == 1 {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 2, transition_sides));
            } else {
                positions.push(regular_position(block, i, j, 0, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, 1, transition_sides));
                positions.push(regular_position(block, i, j, subs - 1, transition_sides));
                positions.push(regular_position(block, i, j, subs - 1, transition_sides));
                positions.push(regular_position(block, i, j, subs, transition_sides));
            }
            // Y-line
            if subs == 1 {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 2, j, transition_sides));
            } else {
                positions.push(regular_position(block, i, 0, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, 1, j, transition_sides));
                positions.push(regular_position(block, i, subs - 1, j, transition_sides));
                positions.push(regular_position(block, i, subs - 1, j, transition_sides));
                positions.push(regular_position(block, i, subs, j, transition_sides));
            }
            // X-line
            if subs == 1 {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
            } else if subs == 2 {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 2, i, j, transition_sides));
            } else {
                positions.push(regular_position(block, 0, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, 1, i, j, transition_sides));
                positions.push(regular_position(block, subs - 1, i, j, transition_sides));
                positions.push(regular_position(block, subs - 1, i, j, transition_sides));
                positions.push(regular_position(block, subs, i, j, transition_sides));
            }
            // High res face lines
            for side in *transition_sides {
                for u_or_v in 0..=(subs * 2) {
                    // U-line
                    positions.push(high_res_face_grid_point_position(
                        block, side, 0, 0, 0, u_or_v,
                    ));
                    positions.push(high_res_face_grid_point_position(
                        block,
                        side,
                        subs - 1,
                        0,
                        2,
                        u_or_v,
                    ));
                    // V-line
                    positions.push(high_res_face_grid_point_position(
                        block, side, 0, 0, u_or_v, 0,
                    ));
                    positions.push(high_res_face_grid_point_position(
                        block,
                        side,
                        0,
                        subs - 1,
                        u_or_v,
                        2,
                    ));
                }
            }
            // Shafts from high-res face points to shrunk regular points
            for i in 0..=block.subdivisions {
                for j in 0..=block.subdivisions {
                    for k in 0..=block.subdivisions {
                        let unshrunk_pos = regular_position(block, i, j, k, &no_side());
                        let actual_pos = regular_position(block, i, j, k, transition_sides);
                        if unshrunk_pos != actual_pos {
                            positions.push(unshrunk_pos);
                            positions.push(actual_pos);
                        }
                    }
                }
            }
            // Indices
            for i in 0..positions.len() {
                indices.push(i as u32);
            }
        }
    }
    let normals = positions.clone(); // Not really important for lines ?
    bevy_mesh.insert_indices(Indices::U32(indices));
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_POSITION, positions);
    bevy_mesh.insert_attribute(BevyMesh::ATTRIBUTE_NORMAL, normals);
    return bevy_mesh;
}

fn high_res_face_grid_point_position(
    block: &Block<f32>,
    side: TransitionSide,
    cell_u: usize,
    cell_v: usize,
    delta_u: usize,
    delta_v: usize,
) -> [f32; 3] {
    let voxel_index = &TransitionCellIndex::from(side, cell_u, cell_v)
        + &HighResolutionVoxelDelta::from(delta_u as isize, delta_v as isize, 0);
    let position_in_block = voxel_index.to_position_in_block(block);
    let pos = &(&position_in_block * block.dims.size) + &block.dims.base;
    [pos.x, pos.y, pos.z]
}

fn regular_position(
    block: &Block<f32>,
    cell_x: usize,
    cell_y: usize,
    cell_z: usize,
    transition_sides: &TransitionSides,
) -> [f32; 3] {
    let cell_size = block.dims.size / block.subdivisions as f32;
    let mut x = block.dims.base[0] + cell_x as f32 * cell_size;
    let mut y = block.dims.base[1] + cell_y as f32 * cell_size;
    let mut z = block.dims.base[2] + cell_z as f32 * cell_size;
    shrink_if_needed::<f32>(
        &mut x,
        &mut y,
        &mut z,
        cell_x as isize,
        cell_y as isize,
        cell_z as isize,
        cell_size,
        block.subdivisions,
        transition_sides,
    );
    [x, y, z]
}

/*

*/

fn append_f3(dest: &mut Vec<[f32; 3]>, src: &VertexAttributeValues, transform: &Transform) -> () {
    if let VertexAttributeValues::Float32x3(values) = src {
        for value in values.iter() {
            let mut new_val = Vec3::from((value[0], value[1], value[2]));
            new_val = transform.transform_point(new_val);
            dest.push([new_val.x, new_val.y, new_val.z]);
        }
    } else {
        panic!()
    }
}

fn append_f2(dest: &mut Vec<[f32; 2]>, src: &VertexAttributeValues) -> () {
    if let VertexAttributeValues::Float32x2(values) = src {
        for value in values.iter() {
            dest.push(*value);
        }
    } else {
        panic!()
    }
}

/*

*/

use std::collections::HashMap;

use noise::{Fbm, NoiseFn, Perlin};
use std::slice::Iter;
use transvoxel::voxel_source::DataField;

#[derive(PartialEq, Debug, Copy, Clone, Hash, Eq)]
pub enum Model {
    Sphere,
    Quadrant,
    Plane,
    Wave,
    Noise,
}

pub fn models_map() -> HashMap<Model, Box<dyn DataField<f32, f32>>> {
    let mut fields: HashMap<Model, Box<dyn DataField<f32, f32>>> = HashMap::new();
    fields.insert(
        Model::Sphere,
        Box::new(Sphere {
            cx: 5f32,
            cy: 5f32,
            cz: 5f32,
            r: 2f32,
        }),
    );
    fields.insert(
        Model::Quadrant,
        Box::new(Sphere {
            cx: 0f32,
            cy: 0f32,
            cz: 0f32,
            r: 6f32,
        }),
    );
    fields.insert(Model::Plane, Box::new(ObliquePlane {}));
    fields.insert(Model::Wave, Box::new(Wave {}));
    fields.insert(Model::Noise, Box::new(Noise::new()));
    return fields;
}

pub const THRESHOLD: f32 = 0.;

impl Model {
    pub fn iterator() -> Iter<'static, Model> {
        static MODELS: [Model; 5] = [
            Model::Sphere,
            Model::Quadrant,
            Model::Plane,
            Model::Wave,
            Model::Noise,
        ];
        MODELS.iter()
    }
}

struct Sphere {
    pub cx: f32,
    pub cy: f32,
    pub cz: f32,
    pub r: f32,
}

impl DataField<f32, f32> for Sphere {
    fn get_data(&mut self, x: f32, y: f32, z: f32) -> f32 {
        let distance_from_center = ((x - self.cx) * (x - self.cx)
            + (y - self.cy) * (y - self.cy)
            + (z - self.cz) * (z - self.cz))
            .sqrt();
        let d = 1f32 - distance_from_center / self.r;
        d
    }
}

struct ObliquePlane {}
impl DataField<f32, f32> for ObliquePlane {
    #[allow(unused_variables)]
    fn get_data(&mut self, x: f32, y: f32, z: f32) -> f32 {
        2f32 + z - 2f32 * y
    }
}

struct Wave {}
impl DataField<f32, f32> for Wave {
    fn get_data(&mut self, x: f32, y: f32, z: f32) -> f32 {
        2.0 * ((x * 1.0).sin() + 0.5 * (z * 0.5).cos()) + 5.0 - y
    }
}

struct Noise {
    f: Box<dyn NoiseFn<f64, 3>>,
}
impl Noise {
    pub fn new() -> Self {
        Self {
            f: Box::new(Fbm::<Perlin>::new(0)),
        }
    }
}
impl DataField<f32, f32> for Noise {
    fn get_data(&mut self, x: f32, y: f32, z: f32) -> f32 {
        let distrub = self.f.get([x as f64, y as f64, z as f64]) as f32;
        2f32 - 2f32 * (y - 3.0 - 3.0 * distrub)
    }
}
