use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::config::PlayerControlConfig;

#[derive(Component)]
pub struct Focus {
    pub point_of_focus: Vec3,
    pub face_direction: Vec3,
    pub free_look: bool,
}

// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn camera_look_system(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
    gamepads: Query<(Entity, &Gamepad)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    config: Res<PlayerControlConfig>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in camera_query.iter_mut() {
            let window_scale = window.height().min(window.width());

            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    let window_scale = window.height().min(window.width());
                    pitch -=
                        (config.mouse_look_sensitivity * accumulated_mouse_motion.delta.y * window_scale)
                            .to_radians();
                    yaw -=
                        (config.mouse_look_sensitivity * accumulated_mouse_motion.delta.x * window_scale)
                            .to_radians();
                    pitch = pitch.clamp(-1.54, 1.54);
                }
            }

            if let Ok((_entity, gamepad)) = gamepads.get_single() {
                let right_stick_x: f32 = gamepad.get(GamepadAxis::RightStickX).unwrap_or_default();
                let right_stick_y: f32 = gamepad.get(GamepadAxis::RightStickY).unwrap_or_default();

                if right_stick_x.abs() > 0.1 {
                    yaw -= (config.gamepad_look_sensitivity * right_stick_x * window_scale).to_radians();
                }

                if right_stick_y.abs() > 0.1 {
                    pitch += (config.gamepad_look_sensitivity * right_stick_y * window_scale).to_radians();
                }
            }

            // prevent the camera from looping over itself in pitch only.
            pitch = pitch.clamp(-1.54, 1.54);
            // Order is important to prevent unintended roll
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}
