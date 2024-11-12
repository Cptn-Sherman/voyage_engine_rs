#![allow(dead_code)]

mod camera;
pub mod config;
mod player;
mod terrain;
mod user_interface;
mod utils;

use avian_interpolation3d::AvianInterpolationPlugin;
use avian_pickup::AvianPickupPlugin;
use bevy::color::palettes::css::YELLOW;
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::VolumetricLight;
use bevy::render::render_asset::RenderAssetBytesPerFrame;

use bevy::render::mesh::Mesh;
use bevy::render::texture::{
    ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasPlugin,
        tonemapping::Tonemapping,
    },
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
use bevy_vector_shapes::prelude::*;

use avian3d::prelude::*;
use bevy_blur_regions::BlurRegionsPlugin;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_kira_audio::{Audio, AudioControl, AudioEasing, AudioPlugin, AudioTween};
use bevy_turborand::prelude::RngPlugin;
use camera::camera::create_camera;
use camera::config::CameraConfig;
use camera::take_screenshot;
use chrono::{DateTime, Local};
use config::{EngineSettings, KeyBindings};
use player::PlayerPlugin;

use std::f32::consts::{FRAC_PI_4, PI};
use std::time::Duration;

use bevy::log::LogPlugin;
use bevy_dev_console::prelude::*;
use user_interface::DebugInterfacePlugin;
use utils::{
    detect_toggle_cursor, generate_plane_mesh,
    increase_render_adapter_wgpu_limits,
};

#[derive(Component)]
struct Sun;

fn main() {
    App::new()
        .init_resource::<InputState>()
        .init_resource::<KeyBindings>()
        .insert_resource(EngineSettings { ..default() })
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .insert_resource(RenderAssetBytesPerFrame::new(1_000_000_000))
        .insert_resource(CameraConfig {
            tonemapping: Tonemapping::Reinhard,
            volumetric_density: 0.0025,
            hdr: true,
        })
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                custom_layer: custom_log_layer,
                ..default()
            }),
            RngPlugin::default(),
            TemporalAntiAliasPlugin,
            DevConsolePlugin,
            Shape2dPlugin::default(),
            // Disabling SyncPlugin is optional, but will get you a performance boost.
            PhysicsPlugins::default(),
            AvianPickupPlugin::default(),
            AvianInterpolationPlugin::default(),
            PhysicsDebugPlugin::default(),
            BlurRegionsPlugin::default(),
            DebugInterfacePlugin,
            PlayerPlugin,
            InfiniteGridPlugin,
            AudioPlugin,
        ))
        .add_systems(
            PreStartup,
            (create_camera, increase_render_adapter_wgpu_limits),
        )
        .add_systems(Startup, (setup, start_background_audio).chain())
        .add_systems(
            Update,
            (
                animate_light_direction,
                detect_toggle_cursor,
                take_screenshot,
            ),
        )
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // ! DO NOT DISTRIBUTE - This music file is for internal testing only. !
    audio
        .into_inner()
        .play(asset_server.load("audio\\liminal-spaces-ambient.ogg"))
        .fade_in(AudioTween::new(
            Duration::from_millis(18000),
            AudioEasing::InPowf(0.125),
        ))
        .with_volume(0.15)
        .looped();
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * 0.0005 * PI / 5.0,
            -FRAC_PI_4 * 0.5,
        );
    }
}

#[derive(Component)]
struct CameraThing;

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
    let plane_thickness: f32 = 0.0001;

    let sampler_desc = ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..Default::default()
    };

    let settings = move |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(sampler_desc.clone());
    };

    let proto_material: Handle<StandardMaterial> = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load_with_settings("textures/proto_dark_01.png", settings.clone()),
        ),
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
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            material: proto_material.clone(),
            ..default()
        },
    ));

    // spawn a ball with physics and a material
    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        Mass(5.0),
        PbrBundle {
            mesh: meshes.add(Sphere::default().mesh().ico(5).unwrap()),
            transform: Transform::from_xyz(2.0, 25.0, 2.0),
            material: materials.add(Color::srgb(0.0, 0.0, 0.9)),
            ..default()
        },
    ));

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
        transform: Transform::from_xyz(-16.0, 2.0, 16.0).with_scale(Vec3 {
            x: 16.0,
            y: 16.0,
            z: 16.0,
        }),
        ..default()
    });
}



#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}
