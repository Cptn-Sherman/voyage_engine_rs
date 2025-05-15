

## Todo:
- Player Controls: 
    - Need a Mechanism for standing up after falling over, and a way to prevent falling over entirely.
        - Detect when the player is not standing upright in Stance
        - Lock Rotation of the player collider in all but the y axis
        - Maybe test unlocking axes of rotation when falling for long enough or getting hit hard enough. aka control lost mode
    - Disable Headbob in Free cam and 3rd-Person
    - Implement Camera Sway in 3rd-Person
    - Feels like you jump really strong and hover, like gravity is added slowly. Probably because we disable gravity for a few ms to allow the jumping.
    - Crouching should be lower but you bottom out and fall over
    - You can full height jump while crouched becuase the ray is short when testing the jump height. You hit 200/200 which feels wrong.
- User Control Scheme:
    - Implement free camera and detach camera from the player entity
    - Implement free camera control and player control. Allowing you to fly away and move the character seperate.
    - Implement a 3rd Person Camera Mode
- Debug Options:
    - Render Avian Colliders
- Interface changes:
    - Make the cursor high-contrast currently its not visisble in bright scenarios... look at minecrafts implementation and probably implement a custom shader or something.
- Setup TOML Configurations:
    - Player Configuration including headbob and camera smoothing
    - Camera Settings
    - Graphical Settings
    - User Interface Themes
- Packages to Checkout in the Future:
    - [Bevy_Mod_Scripting](https://crates.io/crates/bevy_mod_scripting) for Lua Script Support.
    - [Bevy_Tween](https://github.com/djeedai/bevy_tweening) for some Nice Animations.
    - [Bevy_Infinite_Grid](https://github.com/ForesightMiningSoftwareCorporation/bevy_infinite_grid) Infninte Grid Shader.

--- 

## Questions: 
- How can this rust executable know information about what version of bevy it uses
- What version of rust it was compiled with...
- other things
- what os, cpu, gpu, monitor size or window size in non Fullscreen
- is_fullscreen
- How to render a graph of the frame time and update time in ms to show perf
- do I want high and low frames for the past bit of engine running time
- what information is useful in this menu?

Perf Info --> Right Aligned
fps: 165.08 | 6.06 ms/frame
gpu:  22.2% | mem: no_impl
cpu: ---.-% | mem: no_data
entity_count:   -----
chunk_count:    ----- 
updates_queued: -----
draws:  --.--ms
swap:   --.--ms


Graphed Info --> Right Aligned

Engine Info --> Right Aligned, Static
voyage_ver: 0.01.000-ab
build_num:  545625
rust_ver: 1.70.0
bevy_ver: 0.14.0
backend: vulkan

System Info --> Right Aligned, Static (except time)
os: windows_11:24H2
cpu: Intel 3970x 8C-16T
gpu: RTX 3080 10GB
sys_time:  3:42:09pm
last_save: 3:38:59pm <-- set to gold when written and lerps to white as time passes.



World Info <-- Left Aligned
pos:
chunk:
hunk:
loc: [
    Nirn, 
    Tamriel, 
    Morrowind, 
    Balmora, 
    Caius Cosaida's Home
]
biome: liminal
weather: clear
temp: 20.0C | humidity: 20.1%

// we wont add in information that is available using player hud, like hp, mana, stamina, status effects, etc.
Player Info
target:                     <-- what you are looking at
holding:                    <-- what you are holding
    content_id
    instance_id
action:                     <-- what you are doing
animation:                  <-- idk what all will go here but something like this...
    animation_cur1:
    progress:
 
Temporal Info (When) <-- Left Aligned
global_time: 1:35:58pm
local_time:  3:35:58pm
date: Oct 07 2023 | fall
tod: evening | UTC: +02:001




