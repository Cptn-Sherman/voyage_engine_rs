

struct Terrain {
    map: bool, // map of hunk_hash to hunk. Hash is based on the hx, hy.
    name: String,
    size: u32,
}

struct TerrainSettings {
    typ: TerrainType,
}

enum TerrainType {
    Flat,
    Spherical,
}

enum VoxelType {

}

