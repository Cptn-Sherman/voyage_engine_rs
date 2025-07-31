use crate::{input::Input, utils::exp_decay};
use bevy::prelude::*;

use super::Player;

#[derive(Component)]
pub struct Focus;

#[derive(Component)]
pub struct FocusTarget;

// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn camera_look_system(
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    input: Res<Input>,
) {
    for mut cam_transform in camera_query.iter_mut() {
        let (mut camera_yaw, mut camera_pitch, camera_roll) =
            cam_transform.rotation.to_euler(EulerRot::YXZ);

        if keys.pressed(KeyCode::AltLeft) {
            camera_yaw -= (input.focus_delta.x).to_radians();
            info!("Camera Yaw: {}", camera_yaw);
            // need to limit the difference between camera yaw and true yaw.
        } else {
            // lerp the camera yaw back to true yaw.
            camera_yaw = exp_decay(camera_yaw, 0.0, 4.0, time.delta_secs());
        }

        camera_pitch -= (input.focus_delta.y).to_radians();
        // Prevent the Camera from wrapping over itself in pitch only.
        camera_pitch = camera_pitch.clamp(-1.54, 1.54);
        // Order is important to prevent unintended roll.
        cam_transform.rotation =
            Quat::from_euler(EulerRot::default(), camera_yaw, camera_pitch, camera_roll);
    }
}
