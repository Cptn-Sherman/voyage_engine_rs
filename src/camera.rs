use crate::{config::KeyBindings, player::Player};
use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing, motion_blur::MotionBlur, tonemapping::Tonemapping,
    },
    math::Vec3,
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceReflections, VolumetricFog},
    prelude::*,
    utils::default,
};
use bevy_kira_audio::{Audio, AudioControl, AudioSource};

use bevy::{
    input::ButtonInput,
    prelude::{Commands, Entity, KeyCode, Query, Res, With},
    render::view::screenshot::{save_to_disk, Capturing, Screenshot},
    window::{SystemCursorIcon, Window},
    winit::cursor::CursorIcon,
};
use chrono::{DateTime, Local};

use crate::{
    config::EngineSettings,
    utils::{self, get_valid_extension},
};

#[derive(Component)]
pub struct CameraThing;

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

#[derive(Event, Clone)]
pub struct ToggleCameraEvent {
    mode: CameraMode,
}

#[derive(Clone)]
pub enum CameraMode {
    FirstPerson,
    FreeCam,
}

#[derive(Resource)]
pub struct ToggleCameraFreeModeAudioHandle(Handle<AudioSource>);

#[derive(Resource)]
pub struct ToggleCameraFirstModeAudioHandle(Handle<AudioSource>);

pub fn load_toggle_camera_soundfxs(mut commands: Commands, asset_server: Res<AssetServer>) {
    let free_handle = asset_server.load("audio\\Blip-003.wav");
    let first_handle = asset_server.load("audio\\Blip-004.wav");
    commands.insert_resource(ToggleCameraFreeModeAudioHandle(free_handle.clone()));
    commands.insert_resource(ToggleCameraFirstModeAudioHandle(first_handle.clone()));
}

pub fn play_toggle_camera_soundfx(
    mut _ev_footstep: EventReader<ToggleCameraEvent>,
    audio: Res<Audio>,
    free_handle: Res<ToggleCameraFreeModeAudioHandle>,
    first_handle: Res<ToggleCameraFirstModeAudioHandle>,
) {
    let mut should_play: bool = false;
    let mut mode: CameraMode = CameraMode::FreeCam;

    for _ev in _ev_footstep.read() {
        should_play = true;
        mode = _ev.mode.clone();
    }

    if should_play {
        match mode {
            CameraMode::FirstPerson => {
                audio
                    .into_inner()
                    .play(first_handle.0.clone())
                    .with_volume(0.5);
            }
            CameraMode::FreeCam => {
                audio
                    .into_inner()
                    .play(free_handle.0.clone())
                    .with_volume(0.5);
            }
        }
    }
}

pub fn move_fly_camera(
    camera_query: Query<&mut Transform, (With<Camera3d>, Without<FlyCamera>)>,
    mut query: Query<&mut Transform, With<FlyCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    time: Res<Time>,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || query.is_empty()
        || query.iter().len() > 1
    {
        warn!("Player Motion System did not expected 1 camera(s) recieved {}, and 1 player(s) recieved {}. Expect Instablity!", camera_query.iter().len(), query.iter().len());
        return;
    }
    for camera_transform in camera_query.iter() {
        for mut transform in query.iter_mut() {
            let mut movement_vector: Vec3 = Vec3::ZERO.clone();

            let speed_vector: Vec3 = Vec3::from([10.0, 10.0, 10.0]);

            if keys.pressed(key_bindings.move_forward) {
                movement_vector += camera_transform.forward().as_vec3();
            }
            if keys.pressed(key_bindings.move_backward) {
                movement_vector += camera_transform.back().as_vec3();
            }
            if keys.pressed(key_bindings.move_left) {
                movement_vector += camera_transform.left().as_vec3();
            }
            if keys.pressed(key_bindings.move_right) {
                movement_vector += camera_transform.right().as_vec3();
            }

            if keys.pressed(key_bindings.move_ascend) {
                movement_vector += Vec3::Y;
            }

            if keys.pressed(key_bindings.move_descend) {
                movement_vector -= Vec3::Y;
            }


            movement_vector *= speed_vector * time.delta_secs();
            transform.translation += movement_vector;
        }
    }
}

pub fn swap_camera_target(
    mut commands: Commands,
    mut ev_toggle_cam: EventWriter<ToggleCameraEvent>,
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

    if camera_query.is_empty() {
        warn!("Camera Query was empty, cannot swap camera parent target!");
        valid_queries = false;
    }

    if !valid_queries {
        return;
    }

    // this is not safe, should handle none option
    // we first ensure that each of these entities has only one instance
    let player = player_query.iter().next().unwrap();
    let fly_camera = fly_camera_query.iter().next().unwrap();
    let (camera, mut camera_transform, camera_parent) = camera_query.iter_mut().next().unwrap();
    let camera_parent_unwrapped = camera_parent.unwrap();
    // check the camera to see what its parented to.
    // If its parented to the player, then we want to parent it to the fly camera.
    // else it is parented to the fly camera, and we want it parented to the player.
    if **camera_parent_unwrapped == player {
        camera_transform.translation = Vec3::from_array([0.0, 0.0, 0.0]);
        commands.entity(fly_camera).add_children(&[camera]);
        info!("Attached camera to fly_camera entity.");
        ev_toggle_cam.send(ToggleCameraEvent {
            mode: CameraMode::FreeCam,
        });
    } else {
        camera_transform.translation = Vec3::from_array([0.0, 1.0, 0.0]);
        commands.entity(player).add_children(&[camera]);
        info!("Attached camera to player entity.");
        ev_toggle_cam.send(ToggleCameraEvent {
            mode: CameraMode::FirstPerson,
        });
    }
}

/** This system was taken from the screenshot example: https://bevyengine.org/examples/Window/screenshot/ */
pub fn take_screenshot(
    mut commands: Commands,
    settings: Res<EngineSettings>,
    key_bindings: Res<KeyBindings>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(key_bindings.screenshot_key) {
        // get the formated path as string.
        let date: DateTime<Local> = Local::now();
        let formated_date: chrono::format::DelayedFormat<chrono::format::StrftimeItems> =
            date.format("%Y-%m-%d_%H-%M-%S%.3f");
        let path: String = format!(
            "./voyage_screenshot-{}.{}",
            formated_date.to_string(),
            get_valid_extension(
                &settings.screenshot_format,
                utils::ExtensionType::Screenshot
            )
        );

        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn screenshot_saving(
    mut commands: Commands,
    screenshot_saving: Query<Entity, With<Capturing>>,
    windows: Query<Entity, With<Window>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    match screenshot_saving.iter().count() {
        0 => {
            commands.entity(window).remove::<CursorIcon>();
        }
        x if x > 0 => {
            commands
                .entity(window)
                .insert(CursorIcon::from(SystemCursorIcon::Progress));
        }
        _ => {}
    }
}
