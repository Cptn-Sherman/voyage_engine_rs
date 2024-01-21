


struct Planetoid {
    name: String,
    age: f32,
    diameter: f32,
    is_tectonic: bool,
    is_habitable: bool,
}

struct Plate {

}

struct PlanetoidSimulation {
    planetoid: Planetoid,
    plates: Plate,
}

fn load_biome_tables() -> void {

}

/*

*/
fn generate_planetoid_simulation(seed: i32, subdivisions: i32, initial_plate_count: i32) -> PlanetoidSimulation {
    // Select points for initial plates
    // flood fill till not free plate segments are found
    // recenter plate on center of mass.
    // generate random plate movement direction and angular velocity.
}


fn compute_planetoid_simulation_step(sim: PlanetoidSimulation) -> void {
    // start timer
    // move plates
    // compute collision list
    // for each collision update plate values uplift.
    // generate new plate segements at divergent boundaries.
    // recompute plate center of mass?
    // generate random events
        // meteoric impacts
            // terrain disruption
            // mineral deposition
        // volcanic events
            // deposit ash 
            // deposit volcanic rock
    // update oceanic current simulation
    // update wind simulation
    // update moisture simulations
    // update biome deposits
        // organic deposits
        // sedimentary deposits
    // update biomes placements
    // erode
    // complete.
    // end timer, report result.
}

fn bake_planetoid_simulation_state(sim: PlanetoidSimulation) -> void {
    // height maps
    // layer information
    // biome map
    // persistant noise settings and seeds,
}