use bevy::prelude::Resource;

#[derive(Resource)]
pub struct PlayerControlConfig {
    pub (crate) capsule_height: f32,
    pub (crate) ride_height: f32,
    pub (crate) ride_height_step_offset: f32,
    pub (crate) ray_length_offset: f32,
    pub (crate) ride_spring_strength: f32,
    pub (crate) ride_spring_damper: f32,
    pub (crate) stance_lockout: f32,
    pub (crate) jump_strength: f32,
    pub (crate) movement_speed: f32,
    pub (crate) sprint_speed_factor: f32,
    pub (crate) movement_decay: f32,
    pub (crate) mouse_look_sensitivity: f32,
    pub (crate) gamepad_look_sensitivity: f32,
    pub (crate) enable_view_bobbing: bool,
    pub (crate) crouched_height_factor: f32,
}

impl Default for PlayerControlConfig {
    fn default() -> Self {
        Self {
            capsule_height: 1.0,
            ride_height: 1.5,
            ride_height_step_offset: 0.15,
            ray_length_offset: 0.5,
            ride_spring_strength: 3500.0,
            ride_spring_damper: 300.0,
            stance_lockout: 0.25,
            jump_strength: 200.0,
            movement_speed: 10.0,
            sprint_speed_factor: 2.0,
            movement_decay: 0.90,
            mouse_look_sensitivity: 0.00012, // This value was taken from bevy_flycam.
            gamepad_look_sensitivity: 0.0012, // This value was made up by me!
            enable_view_bobbing: true,
            crouched_height_factor: 0.80,
        }
    }
}

pub trait GetDownwardRayLengthMax {
    fn get_downard_ray_length_max(&self) -> f32;
}