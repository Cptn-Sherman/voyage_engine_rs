use bevy::{
    prelude::{
         App, Plugin, Query, Res, ResMut, Resource, Transform, Update, With,
    },
    time::{Time, Timer, TimerMode}, log::{warn, info},
};

use crate::{camera::GameCamera, utils::{format_value_f32}};

pub mod bevy_mesh;
pub mod chunk_mesh;

pub const CHUNK_SIZE_F32: f32 = 16.0;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE_F32 as i32;
pub const CHUNK_SIZE_F32_MIDPOINT: f32 = CHUNK_SIZE_F32 / 2.0;
pub const CHUNK_SIZE_I32_MIDPOINT: i32 = CHUNK_SIZE_F32_MIDPOINT as i32;


/// Converts a coordinate to a chunk coordinate.
///
/// Chunks are square regions in a 2D grid. This function takes a coordinate
/// and returns the corresponding chunk coordinate. The chunk coordinate
/// represents the index of the chunk that contains the given coordinate.
///
/// # Arguments
///
/// * `coord` - The coordinate value to convert.
///
/// # Returns
///
/// The chunk coordinate that corresponds to the given coordinate.
///
/// # Examples
///
/// ```rust
/// let coord = -15;
/// let chunk_coord = convert_to_chunk_coordinate(coord);
/// assert_eq!(chunk_coord, -1);
/// ```
pub fn convert_to_chunk_coordinate(coord: i32) -> i32 {
    if coord < 0 {
        (coord + 1) / (CHUNK_SIZE_F32 as i32) - 1
    } else {
        coord / CHUNK_SIZE_F32 as i32
    }
}

#[derive(Resource)]
pub struct TerrainPlugin;


pub struct Voxel {
    is_occupied: bool,
}

// Define the Octree node
enum OctreeNode {
    // Children nodes and voxel data
    Internal {
        children: [Option<Box<OctreeNode>>; 8],
    },
    Leaf {
        voxel: Voxel,
    },
    Empty,
}

// Define the Octree structure
struct Octree {
    root: OctreeNode,
}

#[derive(Resource)]
pub struct TerrainData {
    octree: Octree
}

#[derive(Resource)]
pub struct LODRecalculateTimer(Timer);

#[derive(Resource)]
pub struct LODPostionTracker {
    cx: i32,
    cy: i32,
    cz: i32,
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        info!("Initializing terrain plugin");
        app.insert_resource(LODRecalculateTimer(Timer::from_seconds(
            3.0 / 8.0, // this value is arbitrary, but it should be a multiple of 1/8th of a second.
            TimerMode::Repeating,
        )))
        .insert_resource(LODPostionTracker {
            cx: 0,
            cy: 0,
            cz: 0,
        })
        .add_systems(Update, check_lod_position);
    }
}

impl LODPostionTracker {
    fn to_string(&self) -> String {
        format!(
            "[{}, {}, {}]",
            format_value_f32(self.cx as f32, None, true),
            format_value_f32(self.cy as f32, None, true),
            format_value_f32(self.cz as f32, None, true)
        )
    }
}

pub fn check_lod_position(
    time: Res<Time>,
    mut timer: ResMut<LODRecalculateTimer>,
    mut tracked_pos: ResMut<LODPostionTracker>,
    camera_query: Query<&Transform, With<GameCamera>>,
) {
    // guard: timer hasn't finished, return early.
    if !timer.0.tick(time.delta()).just_finished() {
        return; 
    }

    // check that only one camera is in the scene, return if this is false.
    if camera_query.iter().len() > 1 {
        warn!("Query found more than one camera! Tracking will not work until resolved.");
        return;    }

    // iterate over each camera and update the tracked position. Expects there to be only one camera in the scene.
    for camera_transform in camera_query.iter() {
        let cur_position = LODPostionTracker {
            cx: convert_to_chunk_coordinate(camera_transform.translation.x as i32),
            cy: convert_to_chunk_coordinate(camera_transform.translation.y as i32),
            cz: convert_to_chunk_coordinate(camera_transform.translation.z as i32),
        };
        //info!("Your position is: [{}]", transform.translation.to_string());
        if cur_position.cx != tracked_pos.cx
            || cur_position.cy != tracked_pos.cy
            || cur_position.cz != tracked_pos.cz
        {
            info!(
                "You moved from chunk: {} to: {}",
                tracked_pos.to_string(),
                cur_position.to_string()
            );
            *tracked_pos = cur_position;
        } else {
            //info!("You did not move! from: {} to: {}", tracked_pos.to_string(), cur_position.to_string());
        }
    }
}