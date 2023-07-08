enum Flow {
    up,
    down,
    north, 
    south, 
    east, 
    west,
}

struct Voxel {
    id: u32,
    flow: Flow,
}