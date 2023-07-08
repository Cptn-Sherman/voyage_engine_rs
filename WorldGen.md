The world must be
spherical

Use tectonic
simulation for terrain generation

Definable biome
definitions

Definable Flora

Creature areas with
ecosystem controller for each area

**Key Questions**

**Why generate terrain using a tectonic plate
simulation?**

Hypothetically
it would be possible to create the world map simply using traditional
procedural noise techniques. The key difference is that the methodology behind
plate tectonics by its nature generates natural landscapes if implemented
correctly. To implement natural landscapes on a macro scale procedural noise
techniques are not sufficient. For each geological structure that is generated
by plate tectonic movements additional processing would need to be added to
implement the same features in a noise-based approach. For example, one feature
that would be difficult to implement using Perlin noise is mid-oceanic ridges.
These ridges are created when divergent plate boundaries are pulled apart
creating new oceanic crust. The crust ages, moving perpendicular to the mid
ocean boundaries increases and the shapes of the continents on either side of
the ridge often reflect that shape. This one feature which is provided freely
with tectonic simulations would require sweeping changes to implement just this
one feature. Generating a realistic and naturally feeling world as a baseline
for the rest of the game is one of the key design goals for Remnants and can
only be accomplished in my opinion through simulation.

**Temperature Simulation:**

- Initialization for temperature simulation sets the temperature for each position based only on latitude.
- Further iterations will sample the wind and oceanic currents to move some temperature around locally.

**Oceanic Current Simulation:**

- Starting at 5 degrees north and south of the equator ocean currents move west until hitting a landmass. When a land mass is hit the currents separate, heading north and south of the equator moving warm water away.
- Approaching 30 degrees north and south all currents flowing north or south begin to move to the east with the wind belt. When these currents hit a land mass, they break into two directions: cold water moving back towards the equator and joining the previous current; the warm water journeys further away from the equator.
- Finally at the 60 degrees north or south mark water moves to the west. Again, as these currents hit land mass the cold-water heads towards the equator and warm water would move higher if possible.
- Water does not redirect at the coastline, rather, it changes direction at continental shelves.

**Precipitation & Moisture Simulation:**

- Precipitation map will be wiped for each iteration as its values are directly derived from the temperature map, Oceanic Current map, and the Wind vector map. Using this information, the amount of water held by the air parcel can be approximated over bodies of water. Wind vectors are sampled to move the parcels and deposit the water amount on the map. This needs to run some number of samples ~100 iterations.
- This map should also contain waterways and natural terrestrial bodies of water.

**Biome Placement:**

Biome
placement is determined using a lookup table detailed in the climate
classification section. Using the latitude and average annual precipitation map
can determine the biome, average annual temperature, and select a latitude
region. Altitude can also be used to promote the biome selected to one higher
up in the table, allowing for snowcapped mountains.

**Hydraulic & Atmospheric Erosion Simulation:**

Hydraulic
erosion will be used to facilitate secondary ore deposits as well as denote the
location of major waterways and bodies of water based on travel frequency
values.

**Historical World Simulation**

Remnants
need to both simulate before the game starts to set the world structures and
create relationships between different factions. As well as continuing to
simulate the inhabitants of the world to keep the towns, cities, and villages
feeling lively. Non-player characters need full schedules, professions, and
needs they need to meet. Perhaps this can be done partially in the background
while the user creates their character.

**Pre-game Historical Simulation:**

The
Historical simulation phase begins as soon as the player finishes generating
the world map and the information is written successfully to the disk.

- **Generate Random Starting Factions -** The first step is, Generating Starting factions which are placed randomly on the map. These contain both the racial factions, and the group factions.
- For n years simulate between (1 - 10) events per faction:
    - Faction population growth
    - Faction population declines
    - Faction separates from initial settlement creating a second faction
    - Faction moves to new location
    - Faction builds a settlement
    - Faction expands a settlement
    - Faction claims a new node to work
    - Faction declares war on another faction
    - Faction fights another faction

**Factions:**

Factions
describe groups of individual people or creatures. These factions can create a
network of relationships between each other and will impact what information
transfers between the two, how their citizens interact with one another and if
they do trade.

- Factions can be used to describe a generic group of individuals, for example a race of people or a culture. As well as non-uniform groups belonging to a pseudo nation. At the start of the game each race’s faction is generated and relationships are generated procedurally influenced by the history simulation. Random non-uniform factions are placed randomly around the map and simulated to generate structures, historical events, and create an interesting world to experience.
- Faction tags contain information about the state of the settlement. These tags are used to generate faction actions.

**Settlements:**

Settlements
describe a single location either currently inhabited by faction(s) or at one
point inhabited and now abandoned. A settlement's data structure contains
information about its layout, pathways, major service buildings, governance
structure, factions residing at the location, and other important information.

- **Settlement failure -** While not frequent it is possible for settlements to fail to establish a proper foothold after settling or after being forced to relocate suddenly.
- Settlements like to form near water sources, or along rivers.
- Settlements need some information about the resource nodes surrounding themselves, used in both economic simulation & the physical structure of the settlement, detailing the shops and services available.
- A settlement can select to build from a set of structures:
    - Subterrain waterworks
    - Water Wells
    - Aqueduct
    - Noria – water well that collects water in buckets and dumps into aqueduct.
    - Bathhouse
    - Cistern
    - Temple
    - Shop
    - Residential buildings
    - Tavern
    - Inn
    - Market Square/Street
    - Barracks
    - Fort
    - Wall
    - Guard tower
    - Guild Hall
    - Castle
    - Water mill
    - Grain mill
    - Barn
    - Warehouse
    - Main Street
    - Docks
    - Shipyard
- Settlements are categorized by size, and the kinds of services located within that settlement. In ascending order these categories are (encampment, village, town, city, and capital).
- Settlements must advance linearly through the different settlement sizes (Encampment, village, town, city, and capital).
- **Encampment: –** a small settlement, usually comprising of makeshift or semi-permanent structures built by a small group or faction. Often created by outlaws or bandits setting up residence away from the view of others. Encampments have the lowest requirements of any settlement category.
    - **Requirements:** Semi-permanent house(s), Communal meeting space (campfire).
    - **Size:** less than 20 people.
- **Village: –** a moderate sized settlement, comprised of groups of farming families.
    - **Requirements:** Permanent house(s), optional tavern, granary, and farmland.
    - **Size:** less than 100 people.
- **Town:** a large-sized settlement.
    - **Requirements:** Permanent structures, taverns, market street/square, shops, military force, walls, guard post, barracks, and service buildings.
    - **Size:** less than 150 people.
- City:
    - **Requirements:** Permanent structures, taverns, market street/square, shops, Aqueducts, bathhouses, waterworks, forts, guild halls, military force, walls, guard post, barracks, and service buildings.
    - **Size:** less than 250 people.
- **Capital:** only one capital is present in a major province.
    - **Requirements:** Permanent structures, taverns, market street/square, shops, Aqueducts, bathhouses, cisterns, waterworks, forts, guild halls, military force, walls, guard post, barracks, castle, and service buildings.
    - **Size:** less than 500 people

**Quests:**

- **Players do not always want to request quests;** they want to stumble into interconnected storylines. Introductions can sometimes be fine as bulletins but that should never be the only path to find such quests. Quests provided by a frequently interacted non-player character feel much more important than a fetch quest on a message board.
- Quests should always be invitations to something rather than a chokehold and a shove into something the player isn't interested in.
- Players will be able to **create and share quest scenario modules**. Quest scenarios can be loaded into any game world and will allow the player to experience new challenges in the same world space. The scenario might set up a castle filled with vampires threatening a nearby town. The player will be able to choose how to handle the situation as the scenarios exist only as quest prompts.
- **Commissions** repeatable requests for materials, supplies, animal parts, minerals, metals, or any other kind of object. More frequently in populated areas, less so in small towns.
- **Quest Arcs / Campaigns** - quest arcs are self-contained story lines or campaigns presented to the player one at a time. Players cannot have a new arc begin until the previous one is completed or failed. If the arc concludes, no matter if it is a positive or negative outcome, it will allow more to appear.
- Procedural Quest Generation Paper [5].

**Starting Scenarios:**

- Players will be able to choose a starting scenario from the character creation screen. This will allow them to decide how their character will be introduced to the world, how or where they will start, and what kind of quests they will be introduced to. Some players may choose to be placed at a random location and start whatever journey they imagine. Players can also specify a small number of events that will occur during the first few levels of play. Maybe the player wishes to meet up with a companion, get involved in a questline, or join an academy.
- **The default starting scenario** is to awaken in a dungeon, or sepulcher to begin their adventure. Cool imagery awakening on an altar with a stream of light filtering through a crack in the door.

**Economics & Markets:**

- Economics & markets need to be represented as an entity.

**Governance Systems:**

- Elder Council -
- Tribal Council -
- Feudalism –

**Judicial Systems:**

- 

**Plates, Rock Generation, Metals, & Minerals**

**Plates Boundaries:**

| Boundary  
  Type | Plate  
  Interaction | Igneous  
  Rock | Generated  
  Structure |
| --- | --- | --- | --- |
| Divergent | Oceanic  
  - Oceanic | Mafic | Oceanic  
  Ridge |
| Divergent | Continental-  
  Continental | Mafic | Continental  
  Rift |
| Convergent | Oceanic  
  - Oceanic | Mafic  
  / Intermediate | Island  
  Arc |
| Convergent | Oceanic  
  - Continental | Felsic | Continental  
  Mountains |
| Convergent | Continental  
  - Continental | None | Mountain  
  Uplift |
| Hot  
  Spot | None | Mafic | Island  
  Chains / Shield Volcanoes |

**Igneous Rock:**

Igneous
rocks are formed from cooling magma, slowly beneath the ground or quickly from
volcanic outflow. There are two categories of igneous rock extrusive that form
above the terrain while intrusive forms beneath the ground. The speed of
cooling affects the crystal sizes that form in the rock, extrusive rocks will
have very small crystal compositions, compared to its intrusive counterpart
containing the same minerals but cooling down over a much longer time.

**Felsic:** (Convergent Oceanic - Continental)

- **Granite** (Intrusive of Rhyolite) Quartz, Potassium feldspar, sodium-rich plagioclase feldspar, minor amphibole, minor muscovite, and minor biotite.
- **Rhyolite** (Extrusive of Granite) Quartz, Potassium feldspar, sodium-rich plagioclase feldspar, minor amphibole, minor muscovite, and minor biotite.
- **Obsidian** (Extrusive Rapid Cooling) Quartz, Potassium feldspar, sodium-rich plagioclase feldspar, hematite, minor amphibole, minor muscovite, and minor biotite.
- **Pumice** (gassy obsidian) (Extrusive Rapid Cooling) Quartz, Potassium feldspar, sodium-rich plagioclase feldspar, hematite, minor amphibole, minor muscovite, and minor biotite.
- **Tuff** Pyroclastic (Felsic, mafic, intermediate, and ultramafic) (Extrusive Rapid Cooling)

**Intermediate:** (Convergent Oceanic - Oceanic)

- **Diorite** (Intrusive of andesite) Amphibole, Sodium and calcium-rich plagioclase feldspar, minor pyroxene, biotite.
- **Andesite** (Extrusive of diorite) Amphibole, Sodium and calcium-rich plagioclase feldspar, minor pyroxene, biotite.

**Mafic:** (Convergent Oceanic - Oceanic or Oceanic
- Continental or (Divergent boundaries)

- **Gabbro** (Intrusive of basalt) Pyroxene, Calcium-rich plagioclase feldspar, minor amphibole, olivine.
- **Basalt** (Extrusive of gabbro) Pyroxene, Calcium-rich plagioclase feldspar, minor amphibole, olivine.

**Sedimentary Rock:**

Sedimentary
rocks are formed from the accumulation of sediments. Creating via compression
of layers of sediment, chemical deposit of dissolved minerals, and organic
compression of marine organisms building up in oxygen poor areas.

**Clastic:**

- **Conglomerate**  Gravel (round)
- **Breccia** Gravel (angular)
- Sandstone - Sand
- Siltstone - Silt
- **Shale** Clay / Mud

**Chemical:**

- **Gypsum rock** made of gypsum
- **Dolomite** Dolomite (warm shallow marine environment)
- **Chert** quartz
- **Flint** quartz
- **Iron Ore** Hematite, Magnetite
- **Limestone** calcite (warm shallow marine environment)
- **Rock Salt** - Halite

**Organic:**

- **Bituminous Coal**  fine-grained organic matter Peat
- **Chalk** microscopic shells and clay
- **Diatomite** microscopic shells

**Metamorphic Rock:**

Metamorphic
rocks are formed from sedimentary or igneous rocks being modified by heat,
pressure, and chemical processes usually while beneath the surface of the
terrain. This class of rock is separated into two varieties; foliated in layers
and non-foliated whose mineral composition does not align no matter the
pressure applied.

**Foliated:**

- Gneiss
    - Parent Rock: Schist
- Schist
    - Parent Rock: Phyllite
- Phyllite
    - Parent Rock: Slate
- Slate
    - Parent Rock: Shale, Mudstone, or Siltstone

**Non-Foliated:**

- Anthracite
    - Parent Rock: Bituminous Coal
- Amphibolite
    - Parent Rock: Basalt, Gabbro
- Hornfels
    - Parent Rock: Shale, Siltstone, Sandstone, Limestone, Dolomite, Basalt, Gabbro, Rhyolite, Granite, Andesite, Schist, Gneiss
- Lapis Lazuli
    - Parent Rock: Limestone, Marble
- Marble
    - Parent Rock: Limestone, Dolomite
- Mariposite
    - Parent Rock: Dolomite, Quartz
- Novaculite
    - Parent Rock: Chert
- Quartzite
    - Parent Rock: Quartz, Sandstone
- Soapstone
    - Parent Rock: Dolomites
- Skarn
    - Parent Rock: Limestone, Dolomite

**Soil Classification:**

Soil
Classification Scheme derived from the USDA NRCS [United States Department of
Agriculture Natural Resources Conservation Service] which details a taxonomy
for classifying soils samples [2]. Providing information about appearance,
composition (sand, clay, silt, gravel, organics), and fertility.

| Name | Fertility | Composition | Appearance |
| --- | --- | --- | --- |
| Alfisols | High | Clay | Dull  
  Brown / Gray |
| Andisols | High | Volcanic  
  Ash, Organic | Brown |
| Aridisols | Low | Clay | Light  
  Brown |
| Entisols | Low | Sand | Light  
  Brown / Orange |
| Gelisols | Moderate | Permafrost,  
  Organic | Dull  
  Brown |
| Histosols | Low | Organic | Dark  
  Brown |
| Inceptisols | High | Mixture | Brown  
  / Pale Brown |
| Mollisols | High | Organic,  
  Clay | Dark  
  Brown |
| Oxisols | Low | Organic | Orange |
| Spodosols | Low | Clay | Brown  
  / Orange |
| Ultisols | Low | Clay | Brown  
  / Orange |
| Vertisols | Moderate | Clay | Very  
  Dark |

**Primary Ore Deposits:**

Primary
ore deposits describe the different ore groups defined by the processes
involved to place them inside the crust. These minerals start in the planet's
magma and are brought higher through magmatic, Hydrothermal, and Submarine
processes. See Figure 1. (Groupings derived from Artifexian)

**Porphyry Deposits**

- **Description:** Magmatic ore deposits that form on the continental overriding plate at convergent subduction zones.
- Major Deposits:
    - Copper, Copper-Gold, Copper-Molybdenum, Molybdenum
- Minor Deposits:
    - Lead, Zinc, Silver, Tin-Tungsten

Epithermal
Gold Deposit

- **Description:** Hydrothermal fluid carries gold from deep in the crust to higher elevation. The process takes place near to continental overriding plates at convergent subduction zones.
- Major Deposits:
    - Gold
- Minor Deposits:
    - Silver, Copper, Lead, Zinc, Mercury

**Iron-Oxide-Copper-Gold (IOCG) Deposit**

- **Description:** Found in older rocks surrounding Epithermal Gold Deposits, however, can also be found in continental rift zones. The divergent boundary between two continental plates.
- Major Deposits:
    - Iron, Copper, Gold
- Minor Deposits:
    - None

**Platinum Group Element (PGE) Deposit**

- **Description:** Hydrothermal deposit found in Ancient Cratons and Old interior continental crust.
- Major Deposits:
    - Nickel-Copper, Platinum, Diamonds
- Minor Deposits:
    - None

**Volcanic Massive Sulfide (VMS) Deposit**

- **Description:** Ore deposits formed from minerals released by black smokers found on ancient sea floors. These deposits are uplifted into both old and new mountains.
- Major Deposits:
    - Copper-Zinc
- Minor Deposits:
    - Lead, Silver, Gold, Cobalt, Tin, Selenium, Manganese, Cadmium

**Banded Iron Formation (BIF) Deposit**

- **Description:** Ancient atmosphere released iron into the ocean which produced layered iron on the seafloor that was later uplifted. These deposits can be found in old mountains.
- Major Deposits:
    - Iron
- Minor Deposits:
    - None

**Sedimentary Exhalative (Sed-Ex) Deposit**

- **Description:** Very uncommon deposits found in continental sedimentary basins.
- Major Deposits:
    - Lead, Zinc, Silver
- Minor Deposits:
    - None

Residual
Mineral Deposit

- **Description:** Intense rainfall leaches minerals from the soil turning it into a pseudo-ore.
- Major Deposits:
    - Aluminum
- Minor Deposits:
    - None

**Secondary Enriched Ore Deposits:**

Secondary
Enriched ore deposits describe the group of ores that were originally deposited
at another location and are relocated by hydraulic processes. See Figure 2.

General
Secondary Deposits

- **Description:** Mineral leaching dissolves mineral deposits and moves them downhill from their source location. The process affects all mineral deposit types.

Mississippi
Valley-Type (MVT) Secondary Deposits

- **Description:** Mineral Leaching dissolves mineral deposits and moves them deep beneath the continental crust surface from mountainous regions to the opposite side of foreland basins, the area surrounding mountain ranges. This movement can surpass 100km of displacement. This process affects all mineral deposit types.

Placer
Secondary Deposits

- **Description:** Mineral outcroppings because of erosion washing mineral particles down river. Commonly gold is washed from its source location and is panned along riverbeds.

**Metals:**

**Pure Metals:**

- Tin, Zinc, Copper, Molybdenum, Gold, Silver, Iron, & Platinum

Fictional
Pure Metals:

- Mythril
    - Description: Fictional Titanium
- Nebulum
    - Description: Colorful metal looks like a nebula
- Abcessium
    - Description: The blood of gods
- Orichalcum
    - Description:
- Pyroclassium
    - Description:
- Glacium
    - Description:
- Luminum
    - Description: Softly glowing pale white metal
- Petrium
    - Description: Formed from Petrified wood

**Metal Alloys:**

- **Bronze -** Combination of Copper and Tin.
- **Brass -** Combination of Copper and Zinc.
- **Steel -** Iron and Carbon
- **Damascus Steel -** Folded steel of different carbon contents.

**Fictional Metal Alloys:**

- **Ember Steel -** Steel & Pyroclassium
    - Ratio: 4:1

**Gemstones:**

- Diamonds:
    - **Variants:** Diamond, Blue Diamond, Brown Diamond, Green Diamond, Red Diamond, Yellow Diamond
    - **Origin:** Mafic, Interior-continental crust, Ancient Cratons
- Beryl:
    - **Variants:** Beryl, Aquamarine, Emerald, Goshenite, Green Beryl, Heliodor, Morganite, Red Beryl
    - **Origin:** Metamorphic stones, rhyolites, limestone, marble
- Opal:
    - **Variants:** Boulder Opal, Cats Eye Opal, Fire Opal, Matrix Opal, Purple Opal, Opalite, Opalized Wood
    - **Origin:** Clastic formed in ancient sedimentary rock that was underwater
- Quartz:
    - **Variants:** Amethyst, Ametrine, Aventurine, Citrine, Prasiolite, Rose Quartz, Smoky Quartz
    - **Origin:** Felsic or Intermediate
- Sapphire:
    - **Origin**: Mafic
- Ruby:
    - **Origin**: Mafic
- Sunstone:
    - **Origin:** Mafic Extrusive
- Moonstone:
    - Fictional
    - **Origin:** Mafic Extrusive
- Wonderstone:
    - **Origin:** Felsic Tuff rock volcanic
- Malachite:
    - **Origin:** Chemical Sedimentary
- Garnet:
    - **Origin:** Felsic, Mafic, and Intermediary
- Topaz:
    - **Origin**: Felsic

**Geological Formations:**

Geological
formations suggest structures that are produced by a geological process.

**Estuarine Coastlines** (Abstract) -

- **Location -** Occurring where rivers or collections of riverways meet the ocean. In this area the fresh and saltwater mix into a brackish solution.

**Drowned River Valley Estuaries** (Estuarine
Coastline) -

- **Formation:** Mouth of river is widened and inundated with sea water.

**Bar - Built Estuaries** (Estuarine Coastline)
-

- **Formation:** Estuary formed from river mouth emptying out into the backside of a barrier island. The area between the terrestrial land and the bar becomes brackish.****

**Tectonic Estuaries** (Estuarine Coastline) -

- **Formation:** Estuaries that form at tectonic plate boundaries as the continental crust sinks from transform boundaries raising and lowering the terrain.

**Fjord Estuaries** (Estuarine Coastline) -

- **Formation:** Formed from the movement of ancient glaciers, a deep U-shaped valley is carved out by slow movements. After the glacier has melted a sill of sediment and rock is left at the mouth of the estuary and the sea water rushes in.

**Glossary**

- **Ore Deposit** - An ore deposit describes a deposit of mineral ore in an amount that is significant enough to turn a profit through mining and smelting processes.
- Continental Rift Zone -
- **Relief** The difference between the highest elevation and the lowest elevation in an area. Areas like grasslands will have lower relief while mountainous cliffs will have much higher relief.
- **Beach** Sandy or Rocky accumulations of sediment were land meets ocean.
- **Spit** Trails of sand that form from water passing around islands offshore. The trails form in the direction of the oceanic current.
- **Lagoon** A small body of water separated from a larger body of water by reefs or barrier islands.
- **Tombolo** - Sand bars connected the beach to small islands or Sea Stacks.
- **Barrier Beach / Islands** Beaches formed parallel to mainland’s forming a waterway between the barrier beaches. Often marshes and wetlands form in this area between the two due to the decreased coastal erosion and currents.
- **Raised Beach** - A former beach lifted due to geologic processes or lowered sea levels.
- **Cliff / Bluff** - Rocky cliffs leading to the beach having a vertical or near vertical face often leading down to a small beach that may be above highest tide or below the highest tide.
- **Platform** Rocky sloped platform from cliff to sea.
- **Cove** Area of shoreline forming a circular or semi-circular shape protected by headlands on either side.
- **Sea Caves / Arches / Stacks / Trunks** - Natural progression of headland erosion as ocean currents carve caves out of the sides. These caves eventually break through forming arches and then collapse to form stacks. After some time, the weather will erode the stack till it is completely submerged.
- **Gorge** Ravine carved by waves and water eroding softer material from cliff faces.
- **Uplift Terraces** - Stair shaped series of platforms leading to the ocean formed by progressive lowering of sea levels or geologic process lifting the coastline.

# **Notes
& References**

- Lauri Viitanen's thesis paper on Physically Based Terrain Generation details some of the algorithms used to handle world terrain generation. As well as showcasing results and performance statistics between various runs of the simulation with differing map sizes and plate counts.
- United States Department of Agriculture Natural Resources Conservation Service documentation details and explains the methodology behind the classification system used to identify soil samples. https://www.nrcs.usda.gov/wps/portal/nrcs/detail/soils/survey/class/taxonomy/?cid=nrcs142p2_053577
- Procedural Planet Generation Technique with irregularly distorted subdivided icosahedron. https://experilous.com/1/blog/post/procedural-planet-generation
- Great article on designing conversation systems. http://www.tads.org/howto/convbkg.htm
- Doran, J. and I. Parberry. “Towards Procedural Quest Generation: A Structural Analysis of RPG Quests.” (2010).