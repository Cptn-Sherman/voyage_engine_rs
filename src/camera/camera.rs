use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing, motion_blur::MotionBlur, tonemapping::Tonemapping,
    },
    math::Vec3,
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceReflections, VolumetricFog},
    prelude::{Camera, Camera3d, ClearColorConfig, Commands, Res, Transform},
    ui::UiAntiAlias,
    utils::default,
};

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
            Camera3d::default(),
            Camera {
                hdr: camera_config.hdr,
                ..default()
            },
            TemporalAntiAliasing { ..default() },
            ScreenSpaceAmbientOcclusion { ..default() },
            ScreenSpaceReflections { ..default() },
            MotionBlur { ..default() },
            Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::ZERO, Vec3::Y),
            Tonemapping::TonyMcMapface,
            CameraThing,
        ))
        .insert(VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        });

    // ******************************************
    // ** This is the 2D camera that will render the 2D UI on top of the 3D world.
    // ** It has an order of 1, so it will render on top of the 3D camera.
    // ** It has no clear color, so it will render on top of the 3D camera without clearing the screen.
    // ** It has anti-aliasing enabled, so the UI will look nice and smooth.
    // ******************************************

    commands.spawn((
        Camera {
            order: 1,
            hdr: camera_config.hdr,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        UiAntiAlias::On,
    ));
}
