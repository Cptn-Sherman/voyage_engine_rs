
pub struct Chunk {
    cx: u32, 
    cy: u32,
    cz: u32,
    weather: u16,
    wind_dir: Vec3,
    biome: String,
    voxels: Vec<Vec<Vec<u32>>>, // 3d Array of voxel id's
}

imple getVoxel for Chunk {}

trait getVoxel() -> u32;