use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::player::motion::Input;

use super::{config::PlayerControlConfig, Player};

#[derive(Component)]
pub struct Focus {
    pub point_of_focus: Vec3,
    pub face_direction: Vec3,
    pub free_look: bool,
}

// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn camera_look_system(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    input: Res<Input>,
) {
    for mut cam_transform in camera_query.iter_mut() {
        let (camera_yaw, mut camera_pitch, camera_roll) =
            cam_transform.rotation.to_euler(EulerRot::YXZ);

        camera_pitch -= (input.direction.y).to_radians();

        // Prevent the Camera from wrapping over itself in pitch only.
        camera_pitch = camera_pitch.clamp(-1.54, 1.54);
        // Order is important to prevent unintended roll.
        cam_transform.rotation =
            Quat::from_euler(EulerRot::default(), camera_yaw, camera_pitch, camera_roll);
    }
}

// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn player_rotation_system(
    mut player_query: Query<&mut Transform, With<Player>>,
    input: Res<Input>,
) {
    for mut player_transform in player_query.iter_mut() {
        let (mut player_yaw, player_pitch, player_roll) =
            player_transform.rotation.to_euler(EulerRot::default());

        player_yaw -= (input.direction.x).to_radians();
        //info!("YAW: {}", player_yaw);
        player_transform.rotation =
            Quat::from_euler(EulerRot::default(), player_yaw, player_pitch, player_roll);
    }
}
