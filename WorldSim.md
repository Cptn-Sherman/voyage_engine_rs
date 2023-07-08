**Tectonic Simulation Parameters:**

- **Subduction scrapping ratio –** percent of mass removed from subducting plate & added to overriding plate edge.
- **Subduction melting duration –** number of ticks it takes for subducted crust columns to start being deleted.
- **Plate overlapping aggregation ratio –** ratio used to signify when one plate overlaps another enough to combine into one.
- **Plate overlapping friction –** friction applied to overlapping plate by subducted plate reducing speed.
- **Percent change overriding edge expansion –** percentage change that the active tectonic plate will expand by one tile as sediment scraps off.
- **Inter-super continental period –** Time between periods of continental congregation and separation.
- **Continental Cracking mantle heat threshold –** threshold for beginning the process of splitting a significantly large plate into two.
- **Continental Oceanic Ratio –** [30%] ratio representing the amount of land over the total surface area of world. The inverse of this value represents the amount of water present over the total surface area of the world.
- **Average inter-super volcanic eruption –** [800,000 years] average time between super volcanic eruptions.
- **Average inter-volcanic eruption –** [60-80 per year / Number of volcanoes] average time between regular volcanic eruptions.
- **Hydrothermal vent boundary range –** range in kilometers from divergent boundaries where hydrothermal vents form.
- **Location of maximum mass subduction –** used as inter-continental source direction (aka where the plates aim for when a super continental convergent period begins).
- **Cosmic body impact frequency –** frequency at which meteors strike the surface.

**World Height Parameters:**

World
height parameters define the distance from the lowest point to the described
feature or structure. These values are modified from actual values to better
fit gameplay and to prevent file sizes from becoming unmanageable.

- **80 km** – Cloud Sim max height (Completely procedural not stored to disk)
- **64 km –** Max Build height
- **48 km –** Max continental height
- **36 km –** Sea level
- **32 km –** Average Ocean depth height
- **20 km –** Maximum Ocean depth
- **02 km –** Highest magma or non-solid material
- **00 km –** Lowest point mostly magma

**Tectonic Simulation:**

- A small 2d array scaled down from the world size is generated for 40,000 x 40,000 km perhaps 4000 x 4000 km or even 400 x 400 km array. The idea is to create a generalized height map that can be combined with procedural generation to produce more realistic terrain, rather than to simulate worlds on unrealistic scales.
- A selection of n points is selected equal to the number of plates. For each of these points one is selected at random and expanded by one random neighboring cell. Possibly a weighted bias could ensure that a few plates are larger in size. This process is repeated until all the grid cells have been added to a plate. Once a plate can no longer expand it is removed from the random chance pool.
- For each plate a center of mass is computed (COM), a center of plate (COP), a direction vector originating from the COM, and a rotation direction and speed from the COP.
- A Perlin noise function is used to establish initial heights for the terrain, all the terrain at this stage is dense oceanic crust.
- A noise function is warped and loaded into another 2D array storing the magma positions. Local maximums are calculated to find suitable positions for hotspot island chains to generate. Hot spots typically last for 100 million years.
- Tectonic simulation will be used to generate Primary ore deposits.
- Massive veins of ore should generate where the inner most voxels are solid metal and the outer is ore in a stone.

**For y million years: Move each plate** by taking
the center of mass, move it to the new location a float x, y then translate
that to grid position and then move it and check for collisions. This will
allow the plates to move in non-grid aligned directions. Keep a list of
collisions to resolve after moving each plate to ensure that the first plate in
the list is not prioritized in collisions. **Iterate
over every plate and move and rotate it: [1]**

- **Convergent Continental & Continental**. If two continental plates collide mountain ridges are created. The plate with less mass will “give mass” to the larger plate. Both lose small amounts of velocity, however the smaller plate losing mass loses force due to mass loss.
- **Convergent Oceanic & Oceanic.** If two oceanic plates collide the denser plate will be subducted, destroying the denser terrain at that position. Creating island arcs.
- **Convergent Continental & Oceanic.** If one oceanic plate and one continental plate collide the oceanic plate will be subducted, destroying the oceanic terrain at that position, and raising up the terrain on the non-subducted plate.
- **Divergent any plate types.** If two plates of any type are moving away from one another a new section of oceanic crust is generated at the empty cell, forming an oceanic ridge or a new body of water where two continental plate’s part.
- Using the local maximums from the magma array island chains are pushed up as the plates move.

At
this point the simulation has computed the following for each position on the
world map: height, annual rainfall, temperature, average wind vector, minerals,
metals, Gemstone, rock composition, material layering, Sediment type, Biome.
This data should be baked and written to disk.

**Algorithm:**

While
seeming complex the algorithm for this plate tectonic simulation is rather
simple.

- Generate the initial plates with a random starting position and generate that initial column.
- Loop and expand a random plate by claiming 1 new column until the full map has been claimed.