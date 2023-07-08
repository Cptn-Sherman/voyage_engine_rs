# Voyage Engine Design Document: 
The Voyage Engine is a game engine written in Rust using the Bevy graphics library (yes I know rustttttt, also I am aware Bevy is experiemental but im fine to work under these restraints). The voyage engine is design to support standard and VR users. Focusing on procedural world simulation, generating new words with content sets. For example a medieval dnd world set could be applied to a fresh planet. Creating new history via simulation or using existing Universes (Tamriel, Azaroth, Pandora, etc). 

No mini-maps or objective markers outside of the map screen. Force players to be present in the world not staring at a point on the screen to get to the “fun stuff”. Having a compass with just North, South, East, West icons are useful though. Keep the player in the world, place larger landmarks in the work spaced out so the player can know where they are in relation to a large mountain, lake, or structure. 

# Side Notes:
- Yeah, I know rust...
- Yes Bevy is experiment/beta/alpha stage. I am fine using it with this understanding that things may not be available or buggy.
This is a "mostly" comprehensive list of features that I hope to deliver for this engine. As you move further down this page the goals will transition from technical challenges more abstract goals targeting gameplay features.
- **The images included in this document are meant to demonstrate
artistic or stylistic elements and are not intended to showcase the final
design. These works unless otherwise specified are not my property and will not be beyond this document.**

# General Features:
- Immersive Sim Gameplay, Deeply integrated player-world interactions
- Initial game starting scenarios
- Large Scale Octree Transvoxel Meshing World structure with Continuous-LOD
- Dynamic Terrain Mesh Decals, Statics & Mesh (Paint, Moss, Dust, Dirt, Blood, Crumbs of rock, etc), (bending, ).
- Vehical/Mount System. Ride creatures, carriages, 
- System Level Mod Support, scripts, plugins, content packs, overhauls, etc. 
- Traditional, XR, VR, Seated, & Roomscale Control Support (Lofty)
- Midway integration for content and media management. (Very Lofty)



# Visuals & Graphics:
## Rendering
- PBR & Pixelated Material Rendering [tutorial](https://www.youtube.com/watch?v=SrgTb-333YA)
- Raytracing, Global Illumination using SSS (Screen Space Surfels) 
- Volumetric fog support, realistic lighting, cascading shadow maps
- Virtual Texturing, Geometry (Meshlets), & Shadows
- Optional Pixelated Shadows
- PBR (Physcial Based Rendering)
- Terrain and Mesh Blending. Loose Sediment voxels cause more blending. 
- https://www.gamedeveloper.com/programming/advanced-terrain-texture-splatting
- https://polycount.com/discussion/181140/unreal-4-terrain-blending-tool-inspired-by-star-wars-battlefront
- Pathtracing or Ray Marching or Ray Tracing: honestly I don't know the difference :/
- Instanced entity rendering
- Triplanar Texture projecting
- Planetary curvature, done in a shader most likely.

## Particle System
- Dust, Rain, Snow, Ash, Leaves, & Gravel
- Physics & Collisions
- Interaction with weather system, primarily the wind information
- Emmsivie particles

## Physics
- [Rapier](https://rapier.rs/): Fast 2D and 3D physics engine for the Rust programming language.
- [Salva](https://salva.rs/): 2D and 3D fluids simulation engine for the Rust programming languag, with couipling with Rapier 
- Softbody physics? not supported in Rapier right now.

## Animation: 
- Inverse Kinematics (IK)
- Facial Animation Morphs
- Physics based animation and reactions to external forces.
- Collision Feedback - When you hit an actor with a heavy thing or weapon they get a physics based push in the direction of the strike. Like the Skyrim Mod precision with a animation pause on impact.
- Seperated body layers, Upper, Lower, Left and Right Limbs. Do so allows each part to be animated independantly and utilize tweening, blending, and IK to merge.
- Skinned Mesh Physics & Deformation

## Audio
- Procedural Audio Synthesis [LINK](https://splice.com/blog/procedural-audio-video-games/)
- Physics based Reverb and Audio transmission [LINK](https://www.youtube.com/watch?v=LY9x_cVfp1Y)
- Enviornmental Sounds - adding sounds like rain hitting leaves in the forest.
- Retro Compressed Audio Filter
- Similar Music to Minecraft, ambient style music.
- 3D sounds & Reverberation based on sampling the voxel space to compute bouncing and position relative to the player / listener.

## Optimization and Mod Pack Assemblage
- Automatically convert raw textures to compressed data using [BC7](https://www.reedbeta.com/blog/understanding-bcn-texture-compression-formats/) or something similar. 
- Mods can contain derivates with pre-computed compression for smaller download size.
- Per Instance Master Manifests, and per module/pack manifest.
- Need a template program which will compile the plugin or mod and create a zip file containg the code and assets. Installation should be drag and drop, auto install dependencies from a central repo.
- Module/Pack ids are limited to 32,768 (u16) and allow for the same number of entries per pack. Resulting in a item id that is u32 with the upper 16 bits identifying the pack, and the lower being the item number.
- With strong validation tools and testings to avoid "silly" issues.

## Actions
- World Interaction - Wanna interact with a door? click the panel open door button next to the door. Want to interact with an object on the ground, hold alt to free the mouse, then drag the mouse over the item. Click to pick up or drag into your inventory?

## Water
- Depth, edge foam effect, & Light Caustics in shallow waters.
- Wind based strength and direction influenced waves -
- **Tidal Simulation** - have values for highest and lowest of both high and low tide.

## Actors
- The Generic Human and how they "generally" respond to stimulus.
- Actors have a level system where the more the player interacts with them the more complex their simulation becomes. 

## Creatures

## Weather Systems

## Objects
- Visual Containers Shelving for small items, racks for armor and tools.

## World Structure/Geometry
- Voxels generated using the Transvoxel Algorithm modified to support HVT (Hierarchel Voxel Transitions)  allowing for smooth "pop-in" free terrain blending. As you move forward in the landscape you do not see a terrain chunk change from LOD N to LOD-N+1 because the positions and normals will interpolate between the lower LOD to the higher.
    - Rust has a crate for the Transvoxel algorithm
    - How can I add in the HVT?
    - can this be a compute shader in bevy? for perf
    - Looks like Texture Arrays are the go to for terrain texturing and blending. Maybe we add a mask in to select the blend type between different points or use a per-texture blend mode. So like stones are solid shapes at the edges and sand and small particles are mixed better
    - LZ4 Compression
- Voxel Fluids or Semi Fluids (water, sand, loose gravel, straw)
- Each voxel type has a set stability value. This value represents how likely a voxel is to dislodge when another entity collides with the voxel. This value allows for landslides to be caused by players walking on overhangs. Certain weather events may increase or decrease a global stability value, for instance, heavy rainfall may make it more likely that a block will slip when stood on by a player.
- Voxels will be an instance of **content data**.
- Sediment type voxels need compacted variants, used when a heavy object impacts the sediment, allowing for compaction. If a large tree topples over the soil will compact to allow the tree to "sink" into the dirt.



# User Interface
- Some of these features of the map should be disabled based on player skills. If you have no information about the area you should not be able to search about it. Maybe you read a book to gain general info or hear rumors or stories from npcs to gain info on an area and update your map.
- Modern, Flat user interface style.
## Inventory
- Things you are wearing, clothing, armor, and weapons on belts
- Coin is in a coin purse if you have on, with "magic weight negation" money you carry outside this pouch will have weight.
- When moving things around the inventory your weight should be shown at the bottom as a gradient bar, as you reach the encumbrence threshold it changes color. If you are holding something show two end points on the line, the current and the if placed weight.
- backpacks create additional inventory space, grid like similar to minecraft or tarkov. with support for mini-cells so you can have 4 small items in a single large cell or can have a multi cell object with takes up multiple cells.
- should play sounds when you move objects in the inventory. (+Immersion)
## Map
- In the style of google maps, modern, flat, simple.
- [Example](https://www.nexusmods.com/skyrimspecialedition/mods/29932) of a simple flat map theme for skyrim. 
- Hide detailed information or complete information about unexplored areas. Fog-Of-War
- Support for flat and spherical world maps
- Map themes for different game vibes
- Right clicking and dragging should show a little line indicating the distance to the dragged point until released. ("why?, ps. read this not rudely" - slightly-high-alex)
## Codex
- A book of information with quick links from other menus, if you click the name of a location on the map you should open the book with that page open.
## QOL (Quality of Life) 
- Photo Mode & Image Library - Images saved in photo mode should appear in a neat catalogue that the user can access from the main menu and possibly the settings menu. May be too much to have the image library visible from both locations.
- In engine information and guide books for mechanics, creatures, foliage, skills, etc. AKA the Codex (see above)
## Controls (Keybindings)
- **Visuals & Graphics** Visual keybinding menu, with dynamically generated icons for in world prompts. I hate that FromSoftware games don't do this 


# Content
## Weapons:
- **Weapons are tools not hardware**, they should feel important to the character instead of disposable.
- **Weapons do not break!** but keeping them sharp and maintained allows them to do more damage. No one likes breaking weapons but providing buffs is a nice incentive to keep your tools sharp and clean.
- You could have occasional temporary breaks which occur when you take massive damage, like a death save while blocking.
- You cannot store voxels in the inventory "raw" they must be inside a container, a bucket/sack/bag/basket what you have but you have to store them like this.
## Armor:
- **Armor and weapon styles are separate from the item level or material level**. Avoid situations where one style of armor is “best in slot” allowing the players to choose the armor and outfits that fit their role-play rather than just the best defense or benefits. Armor styles can have minimum level requirements to encounter but never capped off.
## Combat:
**Combat is a fluid system** in which the player using melee weapons will press the mouse button down and drag the mouse in the direction of attack. This pressing of the mouse will disable or reduce camera movement, so the player does not lose focus on the enemy. The sword strike animation will play in the direction of attack and follow up moves that utilize the momentum and direction of the previous swing will allow for combos. Swinging a sword from right to left then back from left to right will be executed faster and cost less stamina.
## Magic:

# World Structure
## Objects
- **Monetary System** - Each type of coin increases in value by a factor of ten. The smallest unit of value is the copper coin, ten of which equal a silver coin in value. 100 silver coins equal a single gold coin, and 1000 gold coins equal the value of the platinum bullion. These values are usually displayed like this -> **0128.032.11.7** or **0000.000.00.0**. The highest value is still not set in stone.
- **Coin Rods** – Often large sums of money are presented on wood dowels of fixed length matching the inner shape of each coin. The Copper coin is a triangle with a triangular cut out in the center. This triangular peg will slide in the coins equaling an expected value so a peg of twenty Silver or 10 Copper. This can be extended to digital wallets currency.
- **Wallets!** or **Coin Pouches Limits**. Coin pouches almost always have an equivalent exchange and weight negation enchantment applied; however, each one has its limits. Players can place extra money in banks to lighten their loads or purchase better pouches that can carry more coins. Any coins carried that do not fit inside the pouch will be added to the carry weight. Coin pouches that are not sealed are added to the total coin in the players’ inventory, this is done so that deliveries of money stay separate.
    - If you add more coins they are stored in the inventory and have weight now.

# Mechanics

## Immersion
- Immersion can be 

## Skills & Progression
- If you want to gain a new skill you gotta try using it. Your not locked out of something your just bad at it. If you want to learn to lock pick, do it, or practice during rest periods.

- **Leveling System** - When experience is gained it is not immediately applied to the player's accumulated experience, it is withheld until the player sleeps for 8 hours or until the player's rest effect reaches fully rested.
- Players’ **base attributes do not advance**, rather players select permanent traits to advance their characters abilities. Some starting traits could provide more points to place in attributes.
## Inventory
- **Quick Slots** - players will start with 3 quick slots at the beginning of the game. This number can be increased with belts and pouches up to a max of 12.

## Dialogue & Communication
- Players should have access to a text input field to query an extensive set of pre-generated dialogue options. This input should use fuzzy search to provide suggestions that match or are near matches. For example, if the user begins to type “who are…” the input field may suggest some questions like “who are you?” or “tell me about yourself?”. Internally a question like “who are you?” is translated to inquiry->entity->about this may prompt the NPC to tell the player their name, who their family is, what work they do, or even what factions they associate with.
## Crafting
- 
## Professions
- 
## World Editing or Building
- 
## Combat
- 

## Travel and Naviagtion
- **Camps** - Players can set up move-able camps which can contain: a bedroll, tent, campfire, cooking pot, and other things. These will be placed temporarily to allow the player to sleep while travelling. The camp will be rolled up and stay in its last configuration inside a predetermined space based on the camp roll.
- **Passing Time** - Players have the option to pass time by waiting in place or performing actions that take time such as: reading, weapon maintenance, or meal preparation. These tasks can be done at a player’s camp location or at any location with the proper equipment if the player has permission to use the object and the tools necessary. Some towns may consider this loitering if done in the streets so make sure you are in an appropriate location. These tasks can be queued up at camps as actions to complete before sleeping.
- **“Fast Traveling”** players can use a form of fast traveling to move between places quickly. This fast travel requires the player to pay money for transportation, make decisions based on how quickly or how safely they want to travel and decide how they would like to make a camp or if they want to rent an inn when passing through towns. Perhaps a significantly skilled 
- **Rideable Mounts** players can tame, purchase, and ride mounts to traverse in real time or in the form of fast travel. Ground mounts can travel with stops to set up camp or some can pull carriages or wear saddles with coverings depending on their size and power. Also, some mounts are capable of flight, which are not dependent on roadways to travel between points. Mounts also allow saddle bag storage or carriage storage for the player to utilize.
- **Travel Portals** Players that learn the ability to create harmonized portals can make a permanent linkage to travel quickly to the other layers.

## Crime
- - **Crime** works by building up the notoriety of the character as the items that are stolen are noticed. Small items, if not caught in the moment or in possession of unique items, cannot be seized.
- **Large & Recognizable items** can be noticed or searched for by law enforcement.
- **Notoriety does not carry over between factions** unless the factions are close together and have a good relationship or the notoriety level reaches a very high threshold.
- When you commit a crime you gain the RUSTY tag on skills and abilities. This increases the chance of failure and extends the time to preform said actions. This tag/debuff will be removed with use. The length of the incarceration affecting the length and amount it affects your skills.
## Sleep, Camping and Rest
- during gameplay you build up fatige which requires rest else you will pass out.
- When setting up camp you pick a spot, the game fades to black and your camp is set up (time passes but you should be informed of roughly how long it will take). You can select a number of actions to do and time will advance time to the next day, this is a great bathroom break and pause between in game days. If you set up camp while too tired you player may fail to preform actionss. LIke if you try to read before bed you may fall asleep wihtout reading. You can take time to prepare travel meals, sharpen blades, maintain armor and clothing and of course rest your head.
- Hour and a Half play sessions



# Generation & Simulation
## World Generation/Simulation
- Biome Definition File format
## Scenario's
- Users can also share **scenario files** to include in their games, writing custom scripts to start the game. Ranging from random starting locations to more in-depth scenarios that provide an initiating quest arc for the player to explore first. Providing direction for players who do not know how they want their adventure to play out at character creation.
- Listed are a few example scenarios:
    - Select Location from overworld map
    - Select General Location from overworld map
    - Select Starting Gear with implied backstory
    - Pick a "Valuable" starting item or gear piece, maybe its cursed...
    - Begin with quest or quest arc
    - Select a pet/familiar to awaken the player
    - Select starting level and class gear
    - Fated Encounter: Select a companion to encounter shortly after beginning via an initial quest arc.
## Narative/Story Generation
- 
## Weather Simulation
- Unique weather events and disasters can provide simple quests. Help a town prepare for a coming storm. Rescue the people after a flood, etc.

# Thoughts ???:
- Could we have a CPU octree, which just contains collision data aka maybe leafs are just 4 bits and contains info about if its dangerous, solid, semisolid, empty. also if its water. or a fluid Acceleration Structure
- Maybe instead of a death you get three lives which give you a permanent modification to your character while adding lore and back story. Maybe you die in a troll cave you are inprisoned for a while under their watchful eye. Maybe it acts out as a time skip, or text based adventure where you at the end with a trait to pick. 1 negative which is forced, and one positive which is selected.

## Versioning & Updates:
- **Updating & Versioning** - Ability for user to download any version of the game and engine.
- **Versioning Scheme** – Remnant’s version number system contains a major, minor and patch (2.08.05). All updates that change the major or minor version will be flagged as breaking changes. Meaning that there is a high probably that any mods loaded after the main game will no longer work with the newest version. Patches generally should not change any interfaces that mods will rely on, but in the instance that they do, a flag should be set to inform modders that the latest version will break their mods.
- **Assets and Content** can be provided from external jars and are loaded by a class loader. A modding API is exposed providing access to content data, content scripts, assets, and asset loader classes. Each can be extended to provide functionality to the engine. The voyage engine will implement these details and the remnants game jar will utilize the same system to implement its gameplay features. Voyage engine mods must provide a manifest file generated by the voyage engine API which generates information about the assets, asset loaders, content data, and content scripts included in the pack. The manifest file assigns permanent IDs to each item, demarcates version information, lists dependencies including the version number.
- **Content Data** is a type of content that contains only data. For example, a biome definition would be a type of content data. It contains no scripting instructions and simply populates a structure with values and an entry into a look up table to select the biome. This type of data can simply exist as a json structure.
- **Content Scripts** are a type of data that implement custom scripting & behavior beyond just storing values as content data does. This type of data will utilize the class loader to instantiate the objects into the voyage engine.
- **Assets** define data objects like textures, sound files, animations, and fonts.
- **Asset Loaders** define classes that are responsible for reading from disk and storing into the engine, including buffering data into the GPU using voyage engine API.
- Users should have the ability to **attempt to load mods with mismatched version numbers**. They should be warned that the mod may result in unintended consequences and may not be loadable at all.



# References:
- Great Spherical planet tectonic generator/ simulation article. [Experilous: Procedural Planet Generation](https://experilous.com/1/blog/post/procedural-planet-generation)
- Link to Gamasutra article on designing rouge like game systems. [Josh Ge's Blog - How to Make a Roguelike](https://www.gamasutra.com/blogs/JoshGe/20181029/329512/How_to_Make_a_Roguelike.php)
- Desired User interface Font, Montserrat by Google. [Google Font – Montserrat](https://fonts.google.com/specimen/Montserrat#license)
- Desired Font if using pixel art style, Monogram. https://datagoblin.itch.io/monogram
- Perlin Noise functions provided by Open-source Fast Noise Lite. [Auburn/FastNoiseLite: Fast Portable Noise Library - C# C++ C Java HLSL](https://github.com/Auburn/FastNoiseLite)
- Geographical process (Mineral Placement) defined in part by Artifexian. https://www.youtube.com/channel/UCeh-pJYRZTBJDXMNZeWSUVA
- Blue noise
- Netflix Captions Styling for ease of reading. [Timed Text Style Guide: General Requirements – Netflix | Partner Help Center (netflixstudios.com)](https://partnerhelp.netflixstudios.com/hc/en-us/articles/215758617-Timed-Text-Style-Guide-General-Requirements)
- Accessibility font for Dyslexic. [Home | OpenDyslexic](https://opendyslexic.org/)
- Water rendering. [Water Breakdown (fire-face.com)](http://fire-face.com/personal/water/)
- Documentation on openGL. [LearnOpenGL - Introduction](https://learnopengl.com/Introduction)
- Rayleigh surface waves. [Longitudinal and Transverse Wave Motion (psu.edu)](https://www.acs.psu.edu/drussell/Demos/waves/wavemotion.html)
- Hydraulic erosion. [Water Erosion on Heightmap Terrain (ranmantaru.com)](http://ranmantaru.com/blog/2011/10/08/water-erosion-on-heightmap-terrain/)
- Tectonics.js [Tectonics.js - 3d Plate Tectonics in your Web Browser (davidson16807.github.io)](http://davidson16807.github.io/tectonics.js/)