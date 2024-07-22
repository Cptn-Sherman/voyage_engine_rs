#![allow(dead_code)]

mod bevy_mesh;
mod character;
mod terrain;
mod user_interface;
mod utils;
mod player;

use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{VolumetricFogSettings, VolumetricLight};
use bevy::render::render_asset::{RenderAssetBytesPerFrame, RenderAssetUsages};

use bevy::render::mesh::{Indices, Mesh as BevyMesh, PrimitiveTopology, VertexAttributeValues};
use bevy::render::mesh::Mesh;

use bevy::render::render_resource::{AddressMode, SamplerDescriptor};
use bevy::render::renderer::{RenderAdapter, RenderDevice, RenderInstance};
use bevy::render::settings::{WgpuLimits, WgpuSettings};
use bevy::render::texture::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
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

use avian3d::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_kira_audio::{Audio, AudioControl, AudioEasing, AudioPlugin, AudioTween};
use bevy_turborand::prelude::RngPlugin;
use character::{CharacterPlugin};
use chrono::{DateTime, Local};
use light_consts::lux::DIRECT_SUNLIGHT;

use std::f32::consts::{FRAC_PI_4, PI};
use std::time::Duration;

use bevy_mesh::{mesh_for_model, Model};

use crate::utils::CHUNK_SIZE_I32;
use transvoxel::{transition_sides, voxel_source::Block};
use user_interface::DebugInterfacePlugin;
use utils::{format_value_f32, get_valid_extension, CHUNK_SIZE_F32};

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
    pub toggle_sprint: KeyCode,
    pub toggle_grab_cursor: KeyCode,
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
        }
    }
}

fn main() {
    App::new()
        .init_resource::<InputState>()
        .init_resource::<KeyBindings>()
        .insert_resource(EngineSettings { ..default() })
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .insert_resource(RenderAssetBytesPerFrame::new(1_000_000_000))
        .add_plugins((
            DefaultPlugins,
            RngPlugin::default(),
            TemporalAntiAliasPlugin,
            PhysicsPlugins::default(),
            DebugInterfacePlugin,
            CharacterPlugin,
            InfiniteGridPlugin,
            AudioPlugin,
        ))
        .add_systems(
            PreStartup,
            (create_camera, increase_render_adapter_wgpu_limits),
        )
        .add_systems(
            Startup,
            (setup, start_background_audio).chain(),
        )
        .add_systems(
            Update,
            (
                animate_light_direction,
                detect_toggle_cursor,
                screenshot_on_equals,
            ),
        )
        .run();
}

fn increase_render_adapter_wgpu_limits(render_adapter: Res<RenderAdapter>) {
    render_adapter
        .limits()
        .max_sampled_textures_per_shader_stage = 32;
    info!(
        "max_sampled_textures_per_shader_stage is {} ",
        render_adapter
            .limits()
            .max_sampled_textures_per_shader_stage
    );
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // this file is for internal testing only, DO NOT DISTRIBUTE!
    audio
        .into_inner()
        .play(asset_server.load("audio\\liminal-spaces-ambient.ogg"))
        .fade_in(AudioTween::new(
            Duration::from_millis(1500),
            AudioEasing::OutPowi(4),
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
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
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

fn create_camera(mut commands: Commands) {
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
            VolumetricFogSettings {
                density: 0.0025,
                ..Default::default()
            },
            ShadowFilteringMethod::Temporal,
            CameraThing,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());
}

struct TargetDepth(f32);
impl Default for TargetDepth {
    fn default() -> Self {
        TargetDepth(0.006)
    }
}
struct TargetLayers(f32);
impl Default for TargetLayers {
    fn default() -> Self {
        TargetLayers(8.0)
    }
}
struct CurrentMethod(ParallaxMappingMethod);
impl Default for CurrentMethod {
    fn default() -> Self {
        CurrentMethod(ParallaxMappingMethod::Relief { max_steps: 4 })
    }
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // set up the infinite grid with default settings.
    commands.spawn(InfiniteGridBundle::default());

    // create the 'Sun' with volumetric Lighting enabled.
    commands
        .spawn((
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    color: Color::srgb(1.0, 0.92, 0.80),
                    illuminance: 80000.0,
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
        ))
        .insert(VolumetricLight);

    // Plane
    let plane_size: f32 = 128.0;
    let plane_thickness: f32 = 0.5;

    let sampler_desc = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    };

    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
    };

    let proto_material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load_with_settings("textures/proto_dark_01.png", settings.clone())),
        metallic: 0.0,
        alpha_mode: AlphaMode::Opaque,
        unlit: false,
        ..default()
    });

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(plane_size, plane_thickness, plane_size),
        PbrBundle {
            mesh: generate_plane_mesh(&mut meshes, plane_size, plane_size, 1.0 / 16.0),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            material: proto_material.clone(),
            ..default()
        },
    ));

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
        transform: Transform::from_xyz(-16.0, 1.0, 16.0).with_scale(Vec3 {
            x: 16.0,
            y: 16.0,
            z: 16.0,
        }),
        ..default()
    });
}



fn generate_plane_mesh(meshes: &mut ResMut<Assets<Mesh>>, width: f32, length: f32, uv_scale: f32) -> Handle<Mesh> {
    let half_width = width / 2.0;
    let half_length = length / 2.0;

    let vertices = vec![
        // Top face
        ([-half_width, 0.0, half_length], [0.0, 1.0, 0.0], [0.0, uv_scale * length]),   // Top-left
        ([half_width, 0.0, half_length], [0.0, 1.0, 0.0], [uv_scale * width, uv_scale * length]),    // Top-right
        ([half_width, 0.0, -half_length], [0.0, 1.0, 0.0], [uv_scale * width, 0.0]),   // Bottom-right
        ([-half_width, 0.0, -half_length], [0.0, 1.0, 0.0], [0.0, 0.0]),  // Bottom-left
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // top face
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    for (position, normal, uv) in vertices {
        positions.push(position);
        normals.push(normal);
        uvs.push(uv);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::from(positions));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::from(uvs));
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(mesh.with_generated_tangents().expect("Failed to generate tangents for the mesh"))
}

fn grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        // Check if the cursor is already grabbed
        if window.cursor.grab_mode != CursorGrabMode::Locked {
            toggle_grab_cursor(&mut window);
        }
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn detect_toggle_cursor(
    keys: Res<ButtonInput<KeyCode>>,
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

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor.grab_mode {
        CursorGrabMode::None => {
            // Set the cursor position to the center of the window
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
        _ => {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        }
    }
    // set the cursor to the center of the screen.
    let window_width = window.width();
    let window_height = window.height();
    window.set_cursor_position(Some(Vec2::new(window_width / 2.0, window_height / 2.0)));
}

/** This system was yonked from the screenshot example: https://bevyengine.org/examples/Window/screenshot/ */
fn screenshot_on_equals(
    settings: Res<EngineSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    if keys.just_pressed(KeyCode::Equal) {
        // get the formated path as string.
        let date: DateTime<Local> = Local::now();
        let formated_date: chrono::format::DelayedFormat<chrono::format::StrftimeItems> =
            date.format("%Y-%m-%d_%H-%M-%S%.3f");
        let path: String = format!(
            "./voyage_screenshot-{}.{}",
            formated_date.to_string(),
            get_valid_extension(&settings.format, utils::ExtensionType::Screenshot)
        );

        // attempt to save the screenshot to disk and bubble up.
        match screenshot_manager.save_screenshot_to_disk(main_window.single(), path) {
            Ok(_) => info!("Screenshot saved successfully."),
            Err(e) => {
                error!("Failed to save screenshot: {}", e);
            }
        }
    }
}

// This will be read from a toml file in the future.
#[derive(Resource)]
struct EngineSettings {
    format: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings {
            format: "png".to_owned(),
        }
    }
}

#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}
