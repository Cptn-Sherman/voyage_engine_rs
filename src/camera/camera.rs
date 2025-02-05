use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing, motion_blur::MotionBlur, tonemapping::Tonemapping,
    }, math::Vec3, pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceReflections, VolumetricFog}, prelude::*, render::camera, utils::default
};

use crate::{config::KeyBindings, player::Player, CameraThing};

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

#[derive(Component)]
pub struct FlyCamera;

pub fn create_fly_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 5.0, 0.0).looking_to(Vec3::ZERO, Vec3::Y),
        FlyCamera,
    ));
}

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

pub fn swap_camera_target(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    player_query: Query<Entity, With<Player>>,
    fly_camera_query: Query<Entity, With<FlyCamera>>,
    mut camera_query: Query<(Entity, &mut Transform, Option<&Parent>), With<CameraThing>>,
) {

    if !keys.just_pressed(key_bindings.toggle_camera_mode) {
        return;
    }


    let mut valid_queries: bool = true; 
    if player_query.is_empty() {
        warn!("Player Query was empty, cannot swap camera parent target!");
        valid_queries = false;
    }

    if fly_camera_query.is_empty() {
        warn!("Fly Camera Query was empty, cannot swap camera parent target!");
        valid_queries = false;
    }

    if  camera_query.is_empty() {
        warn!("Camera Query was empty, cannot swap camera parent target!");
        valid_queries = false;
    }

    if !valid_queries {
        return;
    }

    // this is not safe, should handle none option
    // we first ensure that each of these entities has only one instance
    let mut player = player_query.iter().next().unwrap();
    let mut fly_camera = fly_camera_query.iter().next().unwrap();
    let (camera, mut camera_transform, camera_parent) = camera_query.iter_mut().next().unwrap();
    let camera_parent_unwrapped = camera_parent.unwrap();
    // check the camera to see what its parented to. 
    // If its parented to the player, then we want to parent it to the fly camera.
    // else it is parented to the fly camera, and we want it parented to the player.
    if **camera_parent_unwrapped == player {
        camera_transform.translation = Vec3::from_array([0.0, 0.0, 0.0]);
        commands.entity(fly_camera).add_children(&[camera]);
        info!("Attached camera to fly_camera entity.");
    } else {
        camera_transform.translation = Vec3::from_array([0.0, 1.0, 0.0]);
        commands.entity(player).add_children(&[camera]);
        info!("Attached camera to player entity.");
    }
}
