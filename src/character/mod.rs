pub mod body;
pub mod focus;
pub mod motion;
pub mod stance;
pub mod states;

use avian3d::prelude::*;
use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::Assets,
    color::Color,
    ecs::event::ManualEventReader,
    hierarchy::{BuildChildren, Parent},
    input::mouse::MouseMotion,
    log::{info, warn},
    math::{Dir3, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    prelude::{
        default, Bundle, Capsule3d, Commands, Component, Entity, IntoSystemConfigs, Query, Res,
        ResMut, Resource, With, Without,
    },
    render::{camera::Camera, mesh::Mesh},
    transform::components::{GlobalTransform, Transform},
};

use body::Body;
use focus::{camera_look_system, Focus};
use motion::{update_player_motion, Motion};
use stance::{
    load_footstep_sfx, lock_rotation, play_footstep_sfx, tick_footstep, update_player_stance, ActionStep, FootstepEvent, Stance, StanceType, ACTION_STEP_DELTA_DEFAULT
};
use states::{crouched::toggle_crouching, grounded::sprinting::toggle_sprint};

use crate::{grab_cursor, player::{config::PlayerControlConfig, handle_pickup_input, spawn_player, Player}, CameraThing};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing Character plugin...");
        app.insert_resource(PlayerControlConfig::default()); // later we will load from some toml file
        app.add_systems(
            Startup,
            (
                spawn_player,
                attached_camera_system,
                grab_cursor,
                load_footstep_sfx,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                update_player_stance,
                toggle_crouching,
                toggle_sprint,
                update_player_motion,
                lock_rotation,
                play_footstep_sfx,
                tick_footstep,
                handle_pickup_input,
                camera_look_system,
            )
                .chain(),
        );
        app.add_event::<FootstepEvent>();
        info!("Actor plugin successfully initialized!");
    }
}



fn attached_camera_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, Without<Camera>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, Option<&Parent>),
        (With<Camera>, With<CameraThing>, Without<Player>),
    >,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        warn!("The camera attach system did not recieve 1 player and 1 camera. Found {} cameras, and {} players", camera_query.iter().len(), player_query.iter().len());
    }

    for (player_entity, _player_transform) in &mut player_query {
        for (camera_entity, mut camera_transform, camera_parent) in &mut camera_query {
            camera_transform.translation = Vec3::from_array([0.0, 1.0, 0.0]);
            if camera_parent.is_none() {
                commands
                    .entity(player_entity)
                    .push_children(&[camera_entity]);
                info!("Attached Camera to player character as child");
            } else {
                info!("Camera parent already exists, will not set player as parent! ");
            }
        }
    }
}
