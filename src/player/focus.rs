use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::{config::PlayerControlConfig, Player};

#[derive(Component)]
pub struct Focus {
    pub point_of_focus: Vec3,
    pub face_direction: Vec3,
    pub free_look: bool,
}

// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn camera_look_system(
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
    gamepads: Query<(Entity, &Gamepad)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    config: Res<PlayerControlConfig>,
) {
    if let Ok(window) = primary_window.single() {
        for mut player_transform in player_query.iter_mut() {
            for mut cam_transform in camera_query.iter_mut() {

                let window_scale = window.height().min(window.width());
                let (mut player_yaw, player_pitch, player_roll) =
                    player_transform.rotation.to_euler(EulerRot::default());
                let (camera_yaw, mut camera_pitch, camera_roll) = 
                    cam_transform.rotation.to_euler(EulerRot::YXZ);
                
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        camera_pitch -= (config.mouse_look_sensitivity
                            * accumulated_mouse_motion.delta.y)
                            .to_radians();
                        player_yaw -= (config.mouse_look_sensitivity
                            * accumulated_mouse_motion.delta.x)
                            .to_radians();
                    }
                }

                if let Ok((_entity, gamepad)) = gamepads.single() {
                    let right_stick_x: f32 =
                        gamepad.get(GamepadAxis::RightStickX).unwrap_or_default();
                    let right_stick_y: f32 =
                        gamepad.get(GamepadAxis::RightStickY).unwrap_or_default();

                    if right_stick_x.abs() > 0.1 {
                        player_yaw -= (config.gamepad_look_sensitivity * right_stick_x * window_scale)
                            .to_radians();
                    }

                    if right_stick_y.abs() > 0.1 {
                        camera_pitch += (config.gamepad_look_sensitivity * right_stick_y * window_scale)
                            .to_radians();
                    }
                }

                // Prevent the Camera from wrapping over itself in pitch only.
                camera_pitch = camera_pitch.clamp(-1.54, 1.54);
                // Order is important to prevent unintended roll.
                cam_transform.rotation = Quat::from_euler(EulerRot::default(), camera_yaw, camera_pitch, camera_roll);
                player_transform.rotation = Quat::from_euler(EulerRot::default(), player_yaw, player_pitch, player_roll);
            }
        }
    }
}
