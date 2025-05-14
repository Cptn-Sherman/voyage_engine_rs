pub mod camera;
pub mod config;
mod player;
mod terrain;
mod user_interface;
mod utils;

use bevy::color::palettes::tailwind::{BLUE_400, RED_900, SKY_400};
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::pbr::{CascadeShadowConfigBuilder, ExtendedMaterial};

use bevy::render::mesh::Mesh;

use bevy::render::render_asset::RenderAssetBytesPerFrame;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};

use avian3d::prelude::*;
use bevy_blockout::{BlockoutMaterialExt, BlockoutPlugin};
use bevy_kira_audio::{Audio, AudioControl, AudioEasing, AudioPlugin, AudioTween};
use bevy_turborand::prelude::RngPlugin;

use camera::{
    create_camera, create_free_camera, load_toggle_camera_soundfxs, move_free_camera,
    play_toggle_camera_soundfx, swap_camera_target, take_screenshot, CameraConfig,
    ToggleCameraEvent,
};
use config::{Bindings, EngineSettings};
use player::PlayerPlugin;

use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

use utils::{detect_toggle_cursor, generate_plane_mesh, increase_render_adapter_wgpu_limits};

#[derive(Component)]
struct Sun;

fn main() {
    App::new()
        .init_resource::<Bindings>()
        .insert_resource(EngineSettings { ..default() })
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .insert_resource(RenderAssetBytesPerFrame::new(2_000_000_000))
        .insert_resource(CameraConfig { hdr: true })
        .add_plugins((
            DefaultPlugins,
            BlockoutPlugin,
            RngPlugin::new().with_rng_seed(0),
            PhysicsPlugins::default(),
            //PhysicsDebugPlugin::default(),
            //DebugInterfacePlugin,
            TemporalAntiAliasPlugin,
            PlayerPlugin,
            AudioPlugin,
        ))
        .add_systems(
            PreStartup,
            (
                create_camera,
                create_free_camera,
                //increase_render_adapter_wgpu_limits,
            ),
        )
        .add_systems(
            Startup,
            (setup, start_background_audio, load_toggle_camera_soundfxs).chain(),
        )
        .add_systems(
            Update,
            (
                //animate_light_direction,
                detect_toggle_cursor,
                swap_camera_target,
                move_free_camera,
                play_toggle_camera_soundfx,
                take_screenshot,
            ),
        )
        .add_event::<ToggleCameraEvent>()
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // ! DO NOT DISTRIBUTE - This music file is for internal testing only!
    audio
        .into_inner()
        .play(asset_server.load("audio/liminal-spaces-ambient.ogg"))
        .fade_in(AudioTween::new(
            Duration::from_millis(18000),
            AudioEasing::InPowf(0.125),
        ))
        .with_volume(0.15)
        .looped();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut extended_materials: ResMut<
        Assets<
            ExtendedMaterial<
                StandardMaterial,
                BlockoutMaterialExt,
            >,
        >,
    >,
) {
    // create the 'Sun' with volumetric Lighting enabled.
    // commands.spawn((
    //     DirectionalLight {
    //         color: Color::srgb(1.0, 0.92, 0.80),
    //         shadows_enabled: true,
    //         shadow_depth_bias: 0.02,
    //         shadow_normal_bias: 1.0,
    //         ..default()
    //     },
    //     VolumetricLight,
    //      Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
    //     Sun,
    // ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.,
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, -FRAC_PI_4)),
    ));

    // Plane
    let plane_size: f32 = 128.0;
    let plane_thickness: f32 = 0.005;

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(plane_size, plane_thickness, plane_size),
        Transform::from_xyz(0.0, 2.0, 0.0),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: SKY_400.into(),
                ..default()
            },
            extension: BlockoutMaterialExt::default(),
        })),
        Mesh3d(generate_plane_mesh(
            &mut meshes,
            plane_size,
            plane_size,
            1.0 / 16.0,
        )),
    ));

    // spawn a ball with physics and a material
    commands.spawn((
        RigidBody::Dynamic,
        Collider::sphere(0.5),
        Mass(5.0),
        Mesh3d(meshes.add(Sphere::default().mesh().ico(5).unwrap())),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.0, 0.9),
            ..default()
        })),
        Transform::from_xyz(2.0, 25.0, 2.0),
    ));
}
