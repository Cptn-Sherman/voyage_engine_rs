use bevy::{input::mouse::MouseMotion, prelude::*, window::{CursorGrabMode, PrimaryWindow}};
use crate::{player::config::PlayerControlConfig, InputState};


#[derive(Component)]
pub struct Focus {
    pub point_of_focus: Vec3,
    pub face_direction: Vec3,
    pub free_look: bool,
}


// This function and many of its helpers are ripped from, bevy_fly_cam.
pub fn camera_look_system(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    motion: Res<Events<MouseMotion>>,
    config: Res<PlayerControlConfig>,
    mut state: ResMut<InputState>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(window) = primary_window.get_single() {
        let delta_state = state.as_mut();
        for mut transform in camera_query.iter_mut() {
            for ev in delta_state.reader_motion.read(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        pitch -= (config.look_sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (config.look_sensitivity * ev.delta.x * window_scale).to_radians();
                        pitch = pitch.clamp(-1.54, 1.54);
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
}