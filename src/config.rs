use bevy::prelude::{KeyCode, Resource};

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_sprint: KeyCode,
    pub toggle_grab_cursor: KeyCode,
    pub interact: KeyCode,
    pub screenshot_key: KeyCode,
    pub toggle_camera_mode: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            toggle_sprint: KeyCode::ShiftLeft,
            toggle_grab_cursor: KeyCode::Escape,
            interact: KeyCode::KeyE,
            screenshot_key: KeyCode::Equal,
            toggle_camera_mode: KeyCode::F3,
        }
    }
}


// This will be read from a toml file in the future.
#[derive(Resource)]
pub struct EngineSettings {
    pub screenshot_format: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings {
            screenshot_format: "png".to_owned(),
        }
    }
}