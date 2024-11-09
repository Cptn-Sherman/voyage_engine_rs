pub mod body;
pub mod focus;
pub mod motion;
pub mod stance;
pub mod states;

use bevy::{
    app::{App, Plugin, Startup, Update},
    hierarchy::{BuildChildren, Parent},
    log::{info, warn},
    math::Vec3,
    prelude::{
        Camera3d, Commands, Entity, IntoSystemConfigs,
        Query, With, Without,
    },
    render::camera::Camera,
    transform::components::Transform,
};

use focus::camera_look_system;
use motion::update_player_motion;
use stance::{
    load_footstep_sfx, lock_rotation, play_footstep_sfx, tick_footstep, update_player_stance,
    FootstepEvent,
};
use states::{crouched::toggle_crouching, grounded::sprinting::toggle_sprint};

use crate::{
    grab_cursor,
    player::{config::PlayerControlConfig, spawn_player, Player},
    CameraThing,
};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing Character plugin...");
        app.insert_resource(PlayerControlConfig::default()); // later we will load from some toml file
        app.add_systems(
            Startup,
            (
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
                camera_look_system,
            )
                .chain(),
        );
        app.add_event::<FootstepEvent>();
        info!("Character plugin successfully initialized!");
    }
}

