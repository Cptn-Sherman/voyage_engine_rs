use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::Resource};


#[derive(Resource)]
pub struct CameraConfig {
    pub(crate) tonemapping: Tonemapping,
    pub(crate) volumetric_density: f32,
    pub(crate) hdr: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            tonemapping: Tonemapping::TonyMcMapface,
            volumetric_density: 0.0025,
            hdr: true,
        }
    }
}