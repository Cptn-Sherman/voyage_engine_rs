mod bevy_mesh;
mod pbr_material;
mod terrain;
mod user_interface;
mod utils;

use bevy::render::mesh::Mesh as BevyMesh;
use bevy::render::mesh::Mesh;

use bevy::{
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{
        DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle,
    },
    prelude::*,
};

use std::f32::consts::{FRAC_PI_4, PI};


use bevy_mesh::{mesh_for_model, Model};
use pbr_material::CustomStandardMaterial;
use terrain::TerrainPlugin;

use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use transvoxel::{transition_sides, voxel_source::Block};
use user_interface::DebugInterfacePlugin;
use utils::{format_value_f32, uv_debug_texture, CHUNK_SIZE_F32, CHUNK_SIZE_F32_MIDPOINT};

use crate::utils::CHUNK_SIZE_I32;

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .add_plugins((
            DefaultPlugins,
            TemporalAntiAliasPlugin,
            DebugInterfacePlugin,
            TerrainPlugin,
            NoCameraPlayerPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            MaterialPlugin::<CustomStandardMaterial>::default(),
        ))
        .add_systems(Startup, (setup, create_voxel_mesh))
        .add_systems(
            Update,
            (
                adjust_directional_light_biases,
                animate_light_direction,
                swap_standard_material,
            ),
        )
        .run();
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
    // Finally, we can run the mesh extraction:
    mesh_for_model(&Model::Noise, false, &block, &transition_sides)
}

pub fn create_voxel_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let radius = 3;
    let length = (radius * 2) + 1;
    let start_x = -radius;
    let start_z = -radius;

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    for i in 0..length {
        for j in 0..length {
            let x = start_x + i;
            let z = start_z + j;
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
                material: debug_material.clone(),
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

fn swap_standard_material(
    mut commands: Commands,
    mut material_events: EventReader<AssetEvent<StandardMaterial>>,
    entites: Query<(Entity, &Handle<StandardMaterial>)>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut custom_materials: ResMut<Assets<CustomStandardMaterial>>,
) {
    for event in material_events.iter() {
        let handle = match event {
            AssetEvent::Created { handle } => handle,
            _ => continue,
        };
        if let Some(material) = standard_materials.get(handle) {
            let custom_mat_h = custom_materials.add(CustomStandardMaterial {
                base_color: material.base_color,
                base_color_texture: material.base_color_texture.clone(),
                emissive: material.emissive,
                emissive_texture: material.emissive_texture.clone(),
                perceptual_roughness: material.perceptual_roughness,
                metallic: material.metallic,
                metallic_roughness_texture: material.metallic_roughness_texture.clone(),
                reflectance: material.reflectance,
                normal_map_texture: material.normal_map_texture.clone(),
                flip_normal_map_y: material.flip_normal_map_y,
                occlusion_texture: material.occlusion_texture.clone(),
                double_sided: material.double_sided,
                cull_mode: material.cull_mode,
                unlit: material.unlit,
                fog_enabled: material.fog_enabled,
                alpha_mode: material.alpha_mode,
                depth_bias: material.depth_bias,
                depth_map: material.depth_map.clone(),
                parallax_depth_scale: material.parallax_depth_scale,
                parallax_mapping_method: material.parallax_mapping_method,
                max_parallax_layer_count: material.max_parallax_layer_count,
            });
            for (entity, entity_mat_h) in entites.iter() {
                if entity_mat_h == handle {
                    let mut ecmds = commands.entity(entity);
                    ecmds.remove::<Handle<StandardMaterial>>();
                    ecmds.insert(custom_mat_h.clone());
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

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
            EnvironmentMapLight {
                diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            },
            FogSettings {
                color: Color::WHITE,
                falloff: FogFalloff::Exponential { density: 0.0001 },
                ..Default::default()
            },
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
            // cascade_shadow_config: CascadeShadowConfigBuilder {
            //     num_cascades: 4,
            //     minimum_distance: 0.01,
            //     maximum_distance: 1024.0,
            //     first_cascade_far_bound: 4.0,
            //     overlap_proportion: 0.2,
            // }
            // .into(),
            ..default()
        },
        Sun,
    ));

    commands.spawn(SceneBundle {
        scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
        transform: Transform::from_xyz(-1.0, 0.0, 0.0),
        ..default()
    });

    // // plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(CHUNK_SIZE_F32_MIDPOINT).into()),
    //     material: materials.add(StandardMaterial {
    //         base_color_texture: Some(base_color_texture),
    //         metallic_roughness_texture: Some(metallic_roughness_texture),
    //         normal_map_texture: Some(normal_map_texture),
    //         //occlusion_texture: Some(occlusion_map_texture),
    //         depth_map: Some(depth_map_texture),
    //         flip_normal_map_y: false,
    //         metallic: 0.95,
    //         ..default()
    //     }),
    //     transform: Transform::from_xyz(CHUNK_SIZE_F32_MIDPOINT, CHUNK_SIZE_F32_MIDPOINT / 2.0, CHUNK_SIZE_F32_MIDPOINT),
    //     ..default()
    // });

    // // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 2500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });

    let corners = [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        (1.0, 1.0, 0.0),
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 1.0),
        (0.0, 1.0, 1.0),
        (1.0, 1.0, 1.0),
    ];

    for corner in corners.iter() {
        let (x, y, z) = *corner;
        let corner_shape_size = 1.0;

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: corner_shape_size,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::ORANGE.into(),
                ..default()
            }),
            transform: Transform::from_xyz(
                x * CHUNK_SIZE_F32,
                y * CHUNK_SIZE_F32 + (corner_shape_size / 2.0),
                z * CHUNK_SIZE_F32,
            ),
            ..default()
        });
    }

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::ORANGE.into(),
            ..default()
        }),
        transform: Transform::from_xyz(CHUNK_SIZE_F32_MIDPOINT, 0.5, CHUNK_SIZE_F32_MIDPOINT),
        ..default()
    });

    let base_color_texture = asset_server.load("textures/dented-metal_albedo.png");
    let metallic_roughness_texture =
        asset_server.load("textures/dented-metal_metallic_roughness_packed.png");
    let normal_map_texture = asset_server.load("textures/dented-metal_normal-ogl.png");
    let depth_map_texture = asset_server.load("textures/dented-metal_height.png");
    // let occlusion_map_texture = asset_server.load("textures/OldIron01_4K_AO.png");

    let mut mesh = Mesh::from(shape::Cube { size: 16.0 });
    mesh.generate_tangents();

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(base_color_texture),
            metallic_roughness_texture: Some(metallic_roughness_texture),
            normal_map_texture: Some(normal_map_texture),
            depth_map: Some(depth_map_texture),
            parallax_depth_scale: 0.05,
            metallic: 0.7,
            reflectance: 0.3,
            perceptual_roughness: 0.3,
            ..default()
        }),
        transform: Transform::from_xyz(
            CHUNK_SIZE_F32_MIDPOINT,
            CHUNK_SIZE_F32_MIDPOINT + 8.0,
            CHUNK_SIZE_F32_MIDPOINT,
        ),
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
        let rotation_speed = 0.5; // Adjust this value to control the rotation speed
        transform.rotate(Quat::from_rotation_x(rotation_speed * time.delta_seconds()));
    }
}
