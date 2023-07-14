mod bevy_mesh;
mod terrain;
mod utils;

use std::f32::consts::PI;

use bevy::render::mesh::Mesh as BevyMesh;
use bevy::render::mesh::Mesh;
use bevy::render::render_resource::SamplerDescriptor;
use bevy::render::texture::ImageSampler;
use bevy::{
    core_pipeline::{
        experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin},
        tonemapping::Tonemapping,
    },
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    pbr::{
        CascadeShadowConfigBuilder, DirectionalLightShadowMap, ScreenSpaceAmbientOcclusionBundle,
    },
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use bevy_mesh::{mesh_for_model, Model};
use terrain::TerrainPlugin;

use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use transvoxel::{transition_sides, voxel_source::Block};
use utils::{format_value, CHUNK_SIZE_F32, CHUNK_SIZE_F32_MIDPOINT};

use crate::utils::CHUNK_SIZE_I32;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TemporalAntiAliasPlugin))
        .add_plugins(TerrainPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, (setup, create_voxel_mesh))
        .add_systems(Update, adjust_directional_light_biases)
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
    .run();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct TpsText;

#[derive(Component)]
struct FrameTimeText;

#[derive(Component)]
struct PosText;

#[derive(Component)]
struct Chunk;

fn build_chunk_mesh(cx: i32, cy: i32, cz: i32) -> BevyMesh {
    let block = Block::from(
        [
            cx as f32 * CHUNK_SIZE_F32, 
            cy as f32 * CHUNK_SIZE_F32, 
            cz as f32 * CHUNK_SIZE_F32
        ],
        CHUNK_SIZE_F32,
        CHUNK_SIZE_I32 as usize,
    );
    let transition_sides = transition_sides::no_side();
    // Finally, we can run the mesh extraction:
    mesh_for_model(&Model::Noise, false, &block, &transition_sides)
}

/// Creates a colorful test pattern
pub fn uv_debug_texture() -> Image {
    info!("Generating Debug Texture");
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    let mut img = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );
    img.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor::default());
    img
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
                "Building Mesh: [{}, {}, {}]",
                format_value(x, None, true),
                format_value(0, None, true),
                format_value(z, None, true)
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
            FogSettings {
                color: Color::WHITE,
                falloff: FogFalloff::Exponential { density: 0.0001 },
                ..Default::default()
            },
            FlyCam,
        ))
        .insert(ScreenSpaceAmbientOcclusionBundle::default())
        .insert(TemporalAntiAliasBundle::default());

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(CHUNK_SIZE_F32).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.05, 0.05, 0.05).into(),
            ..default()
        }),
        transform: Transform::from_xyz(CHUNK_SIZE_F32_MIDPOINT, 0.0, CHUNK_SIZE_F32_MIDPOINT),
        ..default()
    });

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI / 3.,
            -PI / 4.,
        )),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.01,
            maximum_distance: 1024.0,
            first_cascade_far_bound: 4.0,
            overlap_proportion: 0.2,
        }
        .into(),
        ..default()
    });

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

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 10.0,
            sectors: 32,
            stacks: 32,
        })),
        material: debug_material,
        transform: Transform::from_xyz(CHUNK_SIZE_F32_MIDPOINT, 0.5, CHUNK_SIZE_F32_MIDPOINT),
        ..default()
    });

    let default_font_path = "fonts/Pixelme.ttf";


    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_self: AlignSelf::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::FlexStart,
                justify_self: JustifySelf::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Percent(0.25)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgba(0.25, 0.25, 0.25, 0.5)),
            border_color: BorderColor(Color::rgb(0.9, 0.9, 0.9)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "fps: ",
                        TextStyle {
                            font: asset_server.load(default_font_path),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(default_font_path),
                        font_size: 24.0,
                        color: Color::GOLD,
                    }),
                    TextSection::new(
                        "  ",
                        TextStyle {
                            font: asset_server.load(default_font_path),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(default_font_path),
                        font_size: 24.0,
                        color: Color::YELLOW_GREEN,
                    }),
                    TextSection::new(
                        "ms",
                        TextStyle {
                            font: asset_server.load(default_font_path),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                ]),
                FpsText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "tps: ",
                        TextStyle {
                            font: asset_server.load(default_font_path),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(default_font_path),
                        font_size: 24.0,
                        color: Color::PURPLE,
                    }),
                ]),
                TpsText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([]),
                FrameTimeText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "pos: ",
                        TextStyle {
                            font: asset_server.load(default_font_path),
                            font_size: 24.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(default_font_path),
                        font_size: 24.0,
                        color: Color::GOLD,
                    }),
                ]),
                PosText,
            ));
        });
}

fn frame_time_update_system(diagnostics: Diagnostics, mut query: Query<&mut Text, With<FpsText>>) {
    // for mut text in &mut query {
    //     if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
    //         if let Some(value) = fps.smoothed() {
    //             // Update the value of the second section
    //             text.sections[1].value = format!("{value:.2}");
    //         }
    //     }

    //     if let Some(frame_time) = diagnostics.add_measurement(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
    //         if let Some(value) = frame_time.smoothed() {
    //             text.sections[3].value = format!("{value:.2}");
    //         }
    //     }
    // }
}

fn tps_update_system(diagnostics: Diagnostics, mut query: Query<&mut Text, With<TpsText>>) {
    for mut text in &mut query {
        // if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        //     if let Some(value) = fps.smoothed() {
        //         // Update the value of the second section
        //         text.sections[1].value = format!("{value:.2}");
        //     }
        // }
    }
}

fn pos_update_system(
    mut camera_query: Query<(&Camera, &Transform, With<FlyCam>)>,
    mut query: Query<&mut Text, With<PosText>>,
) {
    for (camera, transform, ()) in &mut camera_query {
        for mut text in &mut query {
            text.sections[1].value = format!(
                "[{}, {}, {}]",
                format_value(transform.translation.x, Some(2), true),
                format_value(transform.translation.y, Some(2), true),
                format_value(transform.translation.z, Some(2), true)
            );
        }
    }
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
