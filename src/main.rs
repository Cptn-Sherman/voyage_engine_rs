mod bevy_mesh;
mod terrain;
mod user_interface;
mod utils;
mod character;

use bevy::render::mesh::Mesh as BevyMesh;
use bevy::render::mesh::Mesh;

use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    pbr::{DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle, ShadowFilteringMethod},
    prelude::*,
};

use character::CharacterPlugin;
use chrono::{DateTime, Local};

use bevy_xpbd_3d::components::{Collider, RigidBody};
use bevy_xpbd_3d::plugins::PhysicsPlugins;

use std::f32::consts::{FRAC_PI_4, PI};
use std::time::Duration;

use bevy_mesh::{mesh_for_model, Model};

use crate::utils::CHUNK_SIZE_I32;
use bevy_kira_audio::prelude::*;
use transvoxel::{transition_sides, voxel_source::Block};
use user_interface::DebugInterfacePlugin;
use utils::{format_value_f32, CHUNK_SIZE_F32};

#[derive(Component)]
struct Sun;

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::W,
            move_backward: KeyCode::S,
            move_left: KeyCode::A,
            move_right: KeyCode::D,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}

fn main() {
    color_eyre::install().unwrap();

    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .add_plugins((
            DefaultPlugins,
            TemporalAntiAliasPlugin,
            DebugInterfacePlugin,
            AudioPlugin,
            CharacterPlugin,
            PhysicsPlugins::default(),
        ))
        .add_systems(
            Startup,
            (setup, initial_grab_cursor, start_background_audio),
        )
        .add_systems(Update, (animate_light_direction, screenshot_on_equals, cursor_grab))
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // this file is for internal testing only, DO NOT DISTRIBUTE!
    audio
        .play(asset_server.load("audio\\liminal-spaces-ambient.ogg"))
        .fade_in(AudioTween::new(
            Duration::from_millis(1500),
            AudioEasing::OutPowi(2),
        ))
        .with_volume(0.15)
        .looped();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct Chunk;

fn build_chunk_mesh(cx: i32, cy: i32, cz: i32) -> BevyMesh {
    let block: Block<f32> = Block::from(
        [
            cx as f32 * CHUNK_SIZE_F32,
            cy as f32 * CHUNK_SIZE_F32,
            cz as f32 * CHUNK_SIZE_F32,
        ],
        CHUNK_SIZE_F32,
        CHUNK_SIZE_I32 as usize,
    );
    let transition_sides = transition_sides::no_side();
    mesh_for_model(&Model::Noise, false, &block, &transition_sides)
}

pub fn create_voxel_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = 4;
    let length = radius;
    let start_x = -radius;
    let start_z = -radius;

    // let debug_material = materials.add(StandardMaterial {
    //     base_color_texture: Some(images.add(uv_debug_texture())),
    //     ..default()
    // });

    for i in start_x..length {
        for j in start_z..length {
            let x = i;
            let z = j;
            info!(
                "Building Voxel Mesh @[{}, {}, {}]",
                format_value_f32(x as f32, None, true),
                format_value_f32(0.0, None, true),
                format_value_f32(z as f32, None, true)
            );
            let bevy_mesh = build_chunk_mesh(x, 0, z);
            // This object does not alter the transform as the transvoxel mesh using this information to sample the noise fields.
            commands.spawn(PbrBundle {
                mesh: meshes.add(bevy_mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE.into(),
                    ..default()
                }),
                ..default()
            });
        }
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * 0.05 * PI / 5.0,
            -FRAC_PI_4 * 0.5,
        );
    }
}

#[derive(Component)]
struct CameraThing;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Plane
    let plane_size: f32 = 128.0;
    let plane_thickness: f32 = 0.002;

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(plane_size, plane_thickness, plane_size),
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(plane_size))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
    ));

    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(16.0, 8.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
                tonemapping: Tonemapping::TonyMcMapface,
                ..Default::default()
            },
            FogSettings {
                color: Color::WHITE,
                falloff: FogFalloff::Exponential { density: 0.0005 },
                ..Default::default()
            },
            ShadowFilteringMethod::Jimenez14,
            CameraThing,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());

    // light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 0.96, 0.95),
                shadows_enabled: true,
                shadow_depth_bias: 0.02,
                shadow_normal_bias: 1.0,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                PI / 3.,
                -PI / 4.,
            )),
            ..default()
        },
        Sun,
    ));

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
        transform: Transform::from_xyz(-1.0, 0.0, 0.0),
        ..default()
    });
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
}

fn cursor_grab(
    keys: Res<Input<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        if keys.just_pressed(key_bindings.toggle_grab_cursor) {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/** This system was yonked from the screenshot example: https://bevyengine.org/examples/Window/screenshot/ */
// FIXME: filename timestamp fails if two screenshots occur in the same second, also formatting standards... idk.
fn screenshot_on_equals(
    keys: Res<Input<KeyCode>>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    if keys.just_pressed(KeyCode::Equals) {
        let date: DateTime<Local> = Local::now();
        let formated_date = date.format("%Y-%m-%d_%H-%M-%S");
        let path: String = format!("./voyage_screenshot-{}.png", formated_date.to_string());
        info!("saved screenshot: {}", path);
        screenshot_manager
            .save_screenshot_to_disk(main_window.single(), path)
            .unwrap();
    }
}
