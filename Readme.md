

## Todo: 

- When the user makes changes to the camera settings just kill the world camera entity and do a quick push out and push back in with the last frame before pausing in the back ground.

- Continue implementing the debug interface
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
hunk_count:     ----- 
updates_queued: -----
draws:  --.--ms
swap:   --.--ms


Graphed Info --> Right Aligned

Engine Info --> Right Aligned, Static
voyage_ver: 0.01ab
build_num:  545625
rust_ver: 1.70.0
bevy_ver: 0.14.0
backend: vulkan

System Info --> Right Aligned, Static (except time)
os: windows
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


- When the player is moving foward (possibly running/sprinting) it would be cool to give them a little speed boost when they jump maybe 50% of the movespeed is added to an impulse when you kick off the ground. Could also play a sound effect at this moment to signal the jump has happened. I think this might make the player controller feel less slow. Or maybe even when jumping you build up to a capped 20% speed increase by default. You are letting momentum help you move as long as your directional vector remains in the same direction you should be able to take advantage of thise energy. Maybe this scales with your athletic ability or stat.

