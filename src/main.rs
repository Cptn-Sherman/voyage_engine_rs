mod bevy_mesh;
mod terrain;
mod user_interface;
mod player_controller;
mod utils;

use bevy::render::mesh::Mesh as BevyMesh;
use bevy::render::mesh::Mesh;

use bevy::{
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    pbr::{DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle, ShadowFilteringMethod},
    prelude::*,
};
use bevy_xpbd_3d::components::{RigidBody, Collider};
use bevy_xpbd_3d::plugins::PhysicsPlugins;



use std::f32::consts::{FRAC_PI_4, PI};

use bevy_mesh::{mesh_for_model, Model};
use terrain::TerrainPlugin;

use crate::utils::CHUNK_SIZE_I32;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_kira_audio::prelude::*;
use transvoxel::{transition_sides, voxel_source::Block};
use user_interface::DebugInterfacePlugin;
use player_controller::FirstPersonPlayerControllerPlugin;
use utils::{format_value_f32, CHUNK_SIZE_F32};

#[derive(Resource)]
struct EngineSettings {
    show_debug_hud: bool,
    show_player_controller_raycasts: bool,
}

fn main() {
    color_eyre::install().unwrap();

    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .insert_resource(EngineSettings { show_debug_hud: true, show_player_controller_raycasts: true})
        .add_plugins((
            DefaultPlugins,
            AudioPlugin,
            TemporalAntiAliasPlugin,
            DebugInterfacePlugin,
            TerrainPlugin,
            NoCameraPlayerPlugin,
            PhysicsPlugins::default(),
            FirstPersonPlayerControllerPlugin,

        ))
        .add_systems(Startup, (setup, start_background_audio))
        .add_systems(
            Update,
            (adjust_directional_light_biases, animate_light_direction),
        )
        .run();
}

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio
        .play(asset_server.load("audio/liminal-spaces-ambient-432hz-114635.mp3"))
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
    mut images: ResMut<Assets<Image>>,
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Plane
    let plane_size = 128.0;
    let plane_thickness = 0.002;
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
            FlyCam,
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

fn adjust_directional_light_biases(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut DirectionalLight>,
) {
    let depth_bias_step_size = 0.01;
    let normal_bias_step_size = 0.1;
    for mut light in &mut query {
        if input.just_pressed(KeyCode::Key5) {
            light.shadow_depth_bias -= depth_bias_step_size;
            info!(
                "shadow_depth_bias: {}",
                format!("{:.2}", light.shadow_depth_bias)
            );
        }
        if input.just_pressed(KeyCode::Key6) {
            light.shadow_depth_bias += depth_bias_step_size;
            info!(
                "shadow_depth_bias: {}",
                format!("{:.2}", light.shadow_depth_bias)
            );
        }
        if input.just_pressed(KeyCode::Key7) {
            light.shadow_normal_bias -= normal_bias_step_size;
            info!(
                "shadow_normal_bias: {}",
                format!("{:.2}", light.shadow_normal_bias)
            );
        }
        if input.just_pressed(KeyCode::Key8) {
            light.shadow_normal_bias += normal_bias_step_size;
            info!(
                "shadow_normal_bias: {}",
                format!("{:.2}", light.shadow_normal_bias)
            );
        }
    }
}

#[derive(Component)]
struct Sun;

fn rotate_sun(
    time: Res<Time>,
    mut query: Query<(&mut DirectionalLight, &mut Transform, With<Sun>)>,
) {
    for (mut _light, mut transform, _) in query.iter_mut() {
        // Rotate the sun around the Y-axis
        let rotation_speed = 0.25; // Adjust this value to control the rotation speed
        transform.rotate(Quat::from_rotation_x(rotation_speed * time.delta_seconds()));
    }
}
