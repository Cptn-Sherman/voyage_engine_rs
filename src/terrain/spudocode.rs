struct Plate {
    crust_type: CrustType, // Xc
    crust_thickness: f32, // e
    relief_elevation: f32, //z
    crust_age: f32, //Ao
    local_ridge_direction: Vec3, // r
    orogeny_age: f32, // o
    orogeny_type: u32, // f
    local_fold_direction: Vec3, //Ac
    triangulation: Vec<f32>
}

struct Terranes {
    height: f32,
    stack: Vec<(u16, f32)> // ordered list of tuples sediment type + thickness.
}

enum CrustType {
    Continental,
    Oceanic
}


// Steps 
/*
    Constants:
    SEA_LEVEL = 0km
    TIME_STEP = 2000000 years
    PlANET_RADIUS = 6370km
    MAX_OCEANIC_REDIGE_ELEVATION = -1km
    ABYSSAL_PLAIN_ELVATION = -6km
    OCEANIC_TRENCH_ELEVATION = -10km
    MAX_CONTINENTAL_ALTITUDE = 10km
    SUBDUCTION_DISTANCE = 1800km
    COLLISION_DISTANCE = 4200km
    COLLISION_COEFFICIENT_PER_KM = 0.000013
    MAX_PLATE_SPEED_PER_YEAR = 100mm
    BASE_OCEANIC_ELEVASTION_DAMPENING_PER_YEAR = 0.04mm
    BASE_CONTINENTAL_EROSION_PER_YEAR = 0.03mm
    BASE_SEDIMENT_ACCRETION_PER_YEAR = 0.3mm
    BASE_SUBDUCTION_UPLIFT_PER_YEAR = 0.6mm

    BETA_CONTSTANT = 1.0 subject to change not listed in docks, just a weight to blend.

    Function Curves: 
    height_transfer(val: f32) -> f32 {
        let res = val * val;
        return res;
    }

    RUNTIME CONSTANT
    A0 = Average of plate areas at initilization

    Generate a Icosahedron with a high density of triangles.
    Generete n Plates at randomly selected locations, settings values for Crust Type and applying low frequency noise to terranes.
        Expand these plates to encompas all unclaimed points generated during triangluation.
    Simulate Plate movement taking into account for user input for t years.
        Subduction 
            Oceanic to Oceanic - older plate subsides
            Oceanic to Continental - Oceanic subsides
            Continental to Continental - Promotes to Contentintal Collision when terranes collide, otherwise the older plate subsides.

            Subduction Computations:
                For all points p on contental crust with subducted plate beneath:
                    p.relative_speed = (subducting.speedVec - uplifting.speedVec).normalize
                    p.uplift_amount = BASE_SUBDUCTION_UPLIFT_PER_YEAR * distance_transfer_curve(p.distance_to_subduction_front) * speed_transfer_curve(p.relative_speed) * hieght_transfer_curve()
                    p.z_squgies = (p.relief_elevation - OCEANIC_TRENCH_ELEVATION)/(MAX_CONTINENTAL_ALTITUDE - OCEANIC_TRENCH_ELEVATION)
                    elevation_next = p.elevation + (p.uplift_amount * TIME_STEP);
                    uplifting.fold_dir = uplifting.fold_dir + BETA_CONTSTANT * (subducting.speed - uplifting.speed) * TIME_STEP // this could be done with matrix math.
                Slab Pull:
                    we are changing the plate rotation (w) to point towards the subduction front.
                    w(t + TIME_STEP) = w(t) + E * FOR (int k = 0; k > n - 1; k++)(((Ci X Qk)/(Ci X Qk).normalize)) * TIME_STEP
                    Ci is the centroid position of the plate
                    Qk are the points inside the subduction front aka underneath the overriding plate. Maybe we average this point and aim the w vector to this point.
                    E is less than 1 basically a blending thing. How much does teh slab pull matter


            Continental Collision:
                n = normal to sphere at p
                p = position of plate.
                q = centeroid of the terrane
                r = Rc * SQRT(v(q)/V0) * A/A0    // Power law
                f() = (1 - (x / r)^2)^2
                delta_z(p) = delta_c * A * f(d(p, R))
                z(p, t + TIME_STEP) = z(p, t) + delta_z(p)
                f(p, t + TIME_STEP) = (n X (p - q)/ ((p - q).normalize)) X n
                If close enough remove the terranes from subducting plate and onto overriding
            
            
            Oceanic Crust Generation & Plate Sampling:
                When a plate_hook is ready set its height with this
                alpha = distance_to_rift(plate) / (distance_to_ridge(p) + distance_to_plate(p))
                z(p, t + TIME_STEP) = alpha * interpolation_between_plates() + (1 - alpha) * height of ridge(p, t)
            
            
            Plate Rifting: aka when to rift
            For each Plate
                Generate a number between 0.0f - 1.0f if probabilty is higher cause a rifting event.
                    Probability = x * e ^ -x
                    where x = average_number_of_rifts_in_time_window * crust_type_ratio(plate) * plate.Area / AverageAreaAtInit 

                    Rifting is done by cracking the plate at some point where the mantle is very hot possiblyh and craeting a line between the two halves.
            Contenental Erosion and Oceanic Dampening:
                for each terrane:
                    elevation = elevation - ((1 - (elvation/OCEANIC_TRENCH_ELEVATION)) * BASE_OCEANIC_ELEVASTION_DAMPENING_PER_YEAR * TIME_STEP)
                for each oceanic crust terrane:
                    elevation = elevation - ((elvation/OCEANIC_TRENCH_ELEVATION) * BASE_OCEANIC_ELEVASTION_DAMPENING_PER_YEAR * TIME_STEP)
    
    Apply Amplification aka. post processing and noise to higher detail.
    Save data to disk for fetch later.

    Then we do historical simulation.
 */