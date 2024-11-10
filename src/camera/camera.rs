
use bevy::{core_pipeline::{experimental::taa::TemporalAntiAliasBundle, tonemapping::Tonemapping}, math::Vec3, pbr::{ScreenSpaceAmbientOcclusionBundle, ShadowFilteringMethod, VolumetricFogSettings}, prelude::{Camera, Camera2dBundle, Camera3dBundle, ClearColorConfig, Commands, Res, Transform}, render::camera, utils::default};
use bevy_blur_regions::BlurRegionsCamera;

use crate::CameraThing;

use super::config::CameraConfig;

pub fn create_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    // ******************************************
    // ** This is the main 3D camera that will render the 3D world.
    // ** It has a few extra components to enable some post-processing
    // ** effects like volumetric fog, screen space ambient occlusion, and temporal anti-aliasing.
    // ** It also has a blur regions camera component to enable the blur regions post-processing effect.
    // ******************************************
    commands
        .spawn((
            BlurRegionsCamera::default(),
            Camera3dBundle {
                camera: Camera {
                    hdr: camera_config.hdr,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::TonyMcMapface,
                ..Default::default()
            },
            VolumetricFogSettings {
                density: camera_config.volumetric_density,
                ..Default::default()
            },
            ShadowFilteringMethod::Temporal,
            CameraThing,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());
    


    // ******************************************
    // ** This is the 2D camera that will render the 2D UI on top of the 3D world.
    // ** It has an order of 1, so it will render on top of the 3D camera.
    // ** It has no clear color, so it will render on top of the 3D camera without clearing the screen.
    // ******************************************
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1, 
            hdr: camera_config.hdr,
            clear_color: ClearColorConfig::None, 
            ..default()
        },
        ..default()
    });

}
