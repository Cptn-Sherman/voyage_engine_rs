use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing, motion_blur::MotionBlur, tonemapping::Tonemapping,
    },
    math::Vec3,
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceReflections, VolumetricFog},
    prelude::*,
    utils::default,
};

use crate::CameraThing;

use super::config::CameraConfig;

pub fn create_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    commands
        .spawn((
            Camera3d::default(),
            Camera {
                order: 0,
                hdr: camera_config.hdr,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::ZERO, Vec3::Y),
            Tonemapping::ReinhardLuminance,
            TemporalAntiAliasing { ..default() },
            ScreenSpaceAmbientOcclusion { ..default() },
            ScreenSpaceReflections { ..default() },
            MotionBlur { ..default() },
            CameraThing,
        ))
        .insert(VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        });
}

pub fn create_fly_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0).looking_to(Vec3::ZERO, Vec3::Y),
        FlyCamera,
    ));
}

#[derive(Component)]
pub struct FlyCamera;

// pub fn move_fly_camera(mut query: Query<(&mut Transform, With<FlyCamera>)>, time: Res<Time>) {
//     for (mut transform,) in query.iter_mut() {
//         let mut translation = transform.translation;
//         let mut rotation = transform.rotation;

//         let speed = 10.0;

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::KeyW) {
//             translation += rotation.mul_vec3(Vec3::Z) * speed * time.delta_seconds();
//         }

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::KeyS) {
//             translation -= rotation.mul_vec3(Vec3::Z) * speed * time.delta_seconds();
//         }

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::KeyA) {
//             translation -= rotation.mul_vec3(Vec3::X) * speed * time.delta_seconds();
//         }

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::KeyD) {
//             translation += rotation.mul_vec3(Vec3::X) * speed * time.delta_seconds();
//         }

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::Space) {
//             translation += Vec3::Y * speed * time.delta_seconds();
//         }

//         if bevy::input::keyboard::is_key_pressed(&bevy::input::keyboard::KeyCode::ShiftLeft) {
//             translation -= Vec3::Y * speed * time.delta_seconds();
//         }

//         transform.translation = translation;
//     }
// }
