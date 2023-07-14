use bevy::{
    prelude::{
        info, warn, App, Camera, Plugin, Query, Res, ResMut, Resource, Transform, Update, With,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_flycam::FlyCam;

use crate::utils::{convert_to_chunk_coordinate, format_value};

#[derive(Resource)]
pub struct TerrainPlugin;

#[derive(Resource)]
pub struct TerrainResource {}

#[derive(Resource)]
pub struct LODRecalculateTimer(Timer);

#[derive(Resource)]
pub struct LODPostionTracker {
    cx: i32,
    cy: i32,
    cz: i32,
}

impl LODPostionTracker {
    fn to_string(&self) -> String {
        format!(
            "Camera Position: [{}, {}, {}]",
            format_value::<i32>(self.cx, None, true),
            format_value::<i32>(self.cy, None, true),
            format_value::<i32>(self.cz, None, true)
        )
    }
}

pub fn check_lod_position(
    time: Res<Time>,
    mut timer: ResMut<LODRecalculateTimer>,
    mut tracked_pos: ResMut<LODPostionTracker>,
    camera_query: Query<(&Camera, &Transform, With<FlyCam>)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return; // Timer hasn't just finished, early return
    }

    if camera_query.iter().len() > 1 {
        warn!("Query found more than one camera! Tracking will not work until resolved.");
        return;
    }

    for (_camera, transform, ()) in camera_query.iter() {
        let cur_position = LODPostionTracker {
            cx: convert_to_chunk_coordinate(transform.translation.x as i32),
            cy: convert_to_chunk_coordinate(transform.translation.y as i32),
            cz: convert_to_chunk_coordinate(transform.translation.z as i32),
        };
        //info!("Your position is: [{}]", transform.translation.to_string());
        if cur_position.cx != tracked_pos.cx
            || cur_position.cy != tracked_pos.cy
            || cur_position.cz != tracked_pos.cz
        {
            info!(
                "You moved! from: {} to: {}",
                tracked_pos.to_string(),
                cur_position.to_string()
            );
            *tracked_pos = cur_position;
        } else {
            //info!("You did not move! from: {} to: {}", tracked_pos.to_string(), cur_position.to_string());
        }
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        info!("Initializing terrain plugin");
        app.insert_resource(LODRecalculateTimer(Timer::from_seconds(
            0.5,
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
