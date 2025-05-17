pub mod camera;
pub mod config;
mod player;
mod terrain;
mod user_interface;
mod utils;

use bevy::color::palettes::tailwind::{AMBER_400, SKY_400, ZINC_200};
use bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin;
use bevy::pbr::{CascadeShadowConfigBuilder, ExtendedMaterial};

use bevy::render::mesh::Mesh;

use bevy::render::render_asset::RenderAssetBytesPerFrame;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*};

use avian3d::prelude::*;
use bevy_atmosphere::plugin::AtmospherePlugin;
use bevy_blockout::{BlockoutMaterialExt, BlockoutPlugin};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin};
use bevy_kira_audio::{Audio, AudioControl, AudioEasing, AudioPlugin, AudioTween};
use bevy_sun_move::random_stars::{RandomStarsPlugin, StarSpawner};
use bevy_sun_move::{SkyCenter, SunMovePlugin};
use bevy_turborand::prelude::RngPlugin;

use camera::{
    create_camera, create_free_camera, load_toggle_camera_soundfxs, move_free_camera,
    play_toggle_camera_soundfx, swap_camera_target, take_screenshot, CameraConfig,
    ToggleCameraEvent,
};
use config::{Bindings, EngineSettings};
use player::PlayerPlugin;
use user_interface::DebugInterfacePlugin;

use std::time::Duration;

use utils::{detect_toggle_cursor, generate_plane_mesh};

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
            bevy_panic_handler::PanicHandler::new().build(),
            RngPlugin::new().with_rng_seed(0),
            //TransformInterpolationPlugin::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            DebugInterfacePlugin,
            TemporalAntiAliasPlugin,
            PlayerPlugin,
            AudioPlugin,
            AtmospherePlugin,
            InfiniteGridPlugin,
            BlockoutPlugin,
            SunMovePlugin,
            RandomStarsPlugin,
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
    
    commands.spawn(InfiniteGridBundle::default());

    let _cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();

    // create the 'Sun' with volumetric Lighting enabled.
    let sun_id = commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    )).id();

     commands.spawn((
         SkyCenter {
             sun: sun_id,
             latitude_degrees: 51.5,    // e.g., London's approximate latitude
             planet_tilt_degrees: 23.5, // Earth's axial tilt
             year_fraction: 0.25,       // e.g., Summer Solstice
             cycle_duration_secs: 120.0, // 60-second day/night cycle
             current_cycle_time: 40.0,   // Start at midnight
             ..default()
         },
         Visibility::Visible,
         Transform::default(),
         StarSpawner { star_count: 1000, spawn_radius: 5000.0 }, // Optional
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

    // spawn a cube with physics and a material
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        Mass(5.0),
        Mesh3d(meshes.add(Cuboid::from_length(0.5))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: AMBER_400.into(),
            ..default()
        })),
        Transform::from_xyz(2.0, 25.0, 2.0),
    ));

    // spawn a cube with physics and a material
    let mini_plateform_cube_size: f32 = 2.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(mini_plateform_cube_size, mini_plateform_cube_size, mini_plateform_cube_size),
        Mass(5.0),
        Mesh3d(meshes.add(Cuboid::from_length(mini_plateform_cube_size))),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: ZINC_200.into(),
                ..default()
            },
            extension: BlockoutMaterialExt::default(),
        })),
        Transform::from_xyz(4.0, (mini_plateform_cube_size / 2.0) + 2.0, 8.0),
    ));

    // spawn a cube with physics and a material
    let small_plateform_cube_size: f32 = 4.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(small_plateform_cube_size, small_plateform_cube_size, small_plateform_cube_size),
        Mass(5.0),
        Mesh3d(meshes.add(Cuboid::from_length(small_plateform_cube_size))),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: ZINC_200.into(),
                ..default()
            },
            extension: BlockoutMaterialExt::default(),
        })),
        Transform::from_xyz(8.0, (small_plateform_cube_size / 2.0) + 2.0, 8.0),
    ));

    // spawn a cube with physics and a material
    let medium_plateform_cube_size: f32 = 6.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(medium_plateform_cube_size, medium_plateform_cube_size, medium_plateform_cube_size),
        Mass(5.0),
        Mesh3d(meshes.add(Cuboid::from_length(medium_plateform_cube_size))),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: ZINC_200.into(),
                ..default()
            },
            extension: BlockoutMaterialExt::default(),
        })),
        Transform::from_xyz(16.0, (medium_plateform_cube_size / 2.0) + 2.0, 8.0),
    ));

        // spawn a cube with physics and a material
    let large_plateform_cube_size: f32 = 8.0;
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(large_plateform_cube_size, large_plateform_cube_size, large_plateform_cube_size),
        Mass(5.0),
        Mesh3d(meshes.add(Cuboid::from_length(large_plateform_cube_size))),
        MeshMaterial3d(extended_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: ZINC_200.into(),
                ..default()
            },
            extension: BlockoutMaterialExt::default(),
        })),
        Transform::from_xyz(24.0, (large_plateform_cube_size / 2.0) + 2.0, 8.0),
    ));


}
