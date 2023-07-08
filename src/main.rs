use std::f32::consts::PI;

use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};

const CHUNK_SIZE: f32 = 32.0;
const CHUNK_SIZE_MIDPOINT: f32 = CHUNK_SIZE / 2.0;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_plugin(TerrainPlugin)
        .add_startup_system(setup)
        .add_system(text_update_system)
        .add_system(pos_update_system)
        .insert_resource(DirectionalLightShadowMap { size: 4098 })
        .run();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct MetricText;

#[derive(Component)]
struct TpsText;

#[derive(Component)]
struct PosText;

#[derive(Component)]
struct Chunk;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            tonemapping: Tonemapping::Reinhard,
            ..Default::default()
        },
        FogSettings {
            color: Color::WHITE,
            falloff: FogFalloff::Exponential { density: 1e-3 },
            ..Default::default()
        },
        FlyCam,
    ));

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(CHUNK_SIZE).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.05, 0.05, 0.05).into(),
            ..default()
        }),
        transform: Transform::from_xyz(CHUNK_SIZE_MIDPOINT, 0.0, CHUNK_SIZE_MIDPOINT),
        ..default()
    });
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 25000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x((-PI / 4.0) + (0.1234)),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 200.0,
            ..default()
        }
        .into(),
        ..default()
    });

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
                x * CHUNK_SIZE,
                y * CHUNK_SIZE + (corner_shape_size / 2.0),
                z * CHUNK_SIZE,
            ),
            ..default()
        });
    }

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::GOLD.into(),
            ..default()
        }),
        transform: Transform::from_xyz(CHUNK_SIZE_MIDPOINT, 0.5, CHUNK_SIZE_MIDPOINT),
        ..default()
    });

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(25.0), Val::Percent(25.0)),
                align_self: AlignSelf::FlexStart,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                margin: UiRect::all(Val::Percent(2.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(0.0),
                    left: Val::Percent(25.0 + (25.0 / 2.0)),
                    ..default()
                },
                ..Default::default()
            },
            transform: Transform::from_xyz(0.5, 200.0, 0.0),
            background_color: BackgroundColor(Color::rgba(0.25, 0.25, 0.25, 0.5)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    
                ])
                .with_text_alignment(TextAlignment::Center),
                MetricText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "Fps: ",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoMono/RobotoMono-Bold.ttf"),
                            font_size: 14.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/RobotoMono/RobotoMono-Medium.ttf"),
                        font_size: 14.0,
                        color: Color::GOLD,
                    }),
                    TextSection::new(
                        "Tps: ",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoMono/RobotoMono-Bold.ttf"),
                            font_size: 14.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/RobotoMono/RobotoMono-Medium.ttf"),
                        font_size: 14.0,
                        color: Color::PURPLE,
                    }),
                ]),
                MetricText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    TextSection::new(
                        "pos: ",
                        TextStyle {
                            font: asset_server.load("fonts/RobotoMono/RobotoMono-Bold.ttf"),
                            font_size: 14.0,
                            color: Color::WHITE,
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load("fonts/RobotoMono/RobotoMono-Medium.ttf"),
                        font_size: 14.0,
                        color: Color::GOLD,
                    }),
                ]),
                PosText,
            ));
        });
}

/*
Plugin which creates chunks around a provided x, y, z

- potential improvments:
- cache the position if it hasn't changed cx, cy then skip the update this tick.
- when this is actually doing work, like creating the voxel terrain, we should queue the world generation on another thread, compute on a shader.
*/
// fn create_chunk_plugin(mut query: Query<&mut PbrBundle, With<Chunk>>) {
//     // get the camera position.
//     // using the view distance to check a area around this point. Using a loop update the LOD level of each cube found.
//     // unload any cubes outside this area.
//     // create new cubes in the areas that dont have a cube.
// }

// fn create_chunk(
//     mut commands: &Commands,
//     mut meshes: &ResMut<Assets<Mesh>>,
//     mut materials: &ResMut<Assets<StandardMaterial>>,
//     x: f32,
//     y: f32,
//     z: f32
// ) {
//     commands.spawn(PbrBundle {
//         mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//         material: materials.add(Color::BLUE.into()),
//         transform: Transform::from_xyz(x, y + 0.5, z),
//         ..default()
//     });
// }

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<MetricText>>) {
    for mut text in &mut query {

        // get the value of the text to determine which thing we are editing here. A switch case

        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn format_value(value: f32, decimal_digits: usize, format_negative_space: bool) -> String {
    let num_digits = if value == 0.0 {
        1 // Account for the single digit zero
    } else {
        (value.abs() as f32).log10().floor() as usize + 1 // Calculate the number of digits
    };

    let width = if value >= 0.0 {
        num_digits + decimal_digits // Add one extra space for positive values and decimal digits
    } else {
        num_digits + 1 + decimal_digits // Add two extra spaces for negative values (including the negative sign) and decimal digits
    };

    if format_negative_space && value >= 0.0 {
        format!(
            " {:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits
        )
    } else {
        format!(
            "{:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits
        )
    }
}

fn convert_to_chunk_coordinate(coord: i32) -> i32 {
    if coord < 0 {
        (coord + 1) / (CHUNK_SIZE as i32) - 1
    } else {
        coord / CHUNK_SIZE as i32
    }
}

fn pos_update_system(
    mut cameraQuery: Query<(&FlyCam, &mut Transform)>,
    mut query: Query<&mut Text, With<PosText>>,
) {
    for (_camera, transform) in &mut cameraQuery {
        for mut text in &mut query {
            text.sections[1].value = format!(
                "[{}, {}, {}]",
                format_value(transform.translation.x, 2, true),
                format_value(transform.translation.y, 2, true),
                format_value(transform.translation.z, 2, true)
            );
        }
    }
}

pub struct TerrainPlugin;

#[derive(Resource)]
struct LODRecalculateTimer(Timer);

#[derive(Resource)]
struct CameraPositionTracker {
    cx: i32,
    cy: i32,
    cz: i32,
}

impl CameraPositionTracker {
    fn to_string(&self) -> String {
        format!("Camera Position: [{}, {}, {}]", self.cx, self.cy, self.cz)
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        info!("Initializing terrain plugin");
        app.insert_resource(LODRecalculateTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .insert_resource(CameraPositionTracker {
            cx: 0,
            cy: 0,
            cz: 0,
        })
        .add_system(check_camera_position);
    }
}

fn check_camera_position(
    time: Res<Time>,
    mut timer: ResMut<LODRecalculateTimer>,
    mut tracked_pos: ResMut<CameraPositionTracker>,
    query: Query<(&FlyCam, &mut Transform)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return; // Timer hasn't just finished, early return
    }

    if query.iter().len() > 1 {
        warn!("Query found more than one camera! Tracking will not work until resolved.");
        return;
    }

    for (_camera, transform) in query.iter() {
        let cur_position = CameraPositionTracker {
            cx: convert_to_chunk_coordinate(transform.translation.x as i32),
            cy: convert_to_chunk_coordinate(transform.translation.y as i32),
            cz: convert_to_chunk_coordinate(transform.translation.z as i32),
        };
        //info!("Your position is: [{}]", transform.translation.to_string());
        if cur_position.cx != tracked_pos.cx
            || cur_position.cy != tracked_pos.cy
            || cur_position.cz != tracked_pos.cz
        {
            info!(
                "You moved! from: {} to: {}",
                tracked_pos.to_string(),
                cur_position.to_string()
            );
            *tracked_pos = cur_position;
        } else {
            //info!("You did not move! from: {} to: {}", tracked_pos.to_string(), cur_position.to_string());
        }
    }
}
