// basic setup for bevy plugin
use bevy::{prelude::*, diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}};
use bevy_flycam::FlyCam;

use crate::utils::format_value_f32;

const DEFAULT_FONT_PATH: &str = "fonts/Monocraft.ttf";
const DEFAULT_FONT_SIZE: f32 = 12.0;

pub struct DebugInterfacePlugin;

impl Plugin for DebugInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_debug_interface);
        app.insert_resource(FPSUpdateUITimer(Timer::from_seconds(1.0, TimerMode::Repeating)));
        app.add_systems(Update, (frame_time_update_system, tps_update_system, pos_update_system));
    }
}

pub fn create_debug_interface(    
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // setup the default font
    let default_font = asset_server.load(DEFAULT_FONT_PATH);

    // Spawn in the crosshair
    let cursor_size: f32 = 6.0;
    let cursor_color: BackgroundColor = BackgroundColor(Color::WHITE);
    let crosshair_texture_handle = asset_server.load("textures/white_square_crosshair.png");

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                top: Val::Percent(50.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(cursor_size),
                        height: Val::Px(cursor_size),
                        ..default()
                    },
                    // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                    background_color: cursor_color.into(),
                    ..default()
                },
                UiImage::new(crosshair_texture_handle.into()),
            ));
        });


        commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Px(5.0)),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(4.0)),
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
                    gen_text_section(Some("fps: ".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                    gen_text_section(Some("0".to_string()), Some(12.0), Some(Color::YELLOW_GREEN), default_font.clone()),
                    gen_text_section(Some("  ".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                    gen_text_section(Some("0".to_string()), Some(12.0), Some(Color::YELLOW_GREEN), default_font.clone()),
                    gen_text_section(Some(" ms/frame".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                ]),
                FpsText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    gen_text_section(Some("tps: ".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                    gen_text_section(Some("0".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                ]),
                TpsText,
            ));
        })
        .with_children(|parent| {
            parent.spawn((
                // Create a TextBundle that has a Text with a list of sections.
                TextBundle::from_sections([
                    gen_text_section(Some("pos: ".to_string()), Some(12.0), Some(Color::WHITE), default_font.clone()),
                    TextSection::from_style(TextStyle {
                        font: asset_server.load(DEFAULT_FONT_PATH),
                        font_size: DEFAULT_FONT_SIZE,
                        color: Color::GOLD,
                    }),
                ]),
                PosText,
            ));
        });

}

fn gen_text_section(value: Option<String>, size: Option<f32>, color: Option<Color>, font: Handle<Font>) -> TextSection { 
    TextSection::new(
        value.unwrap_or("".to_string()),
        TextStyle {
            font,
            font_size: size.unwrap_or(DEFAULT_FONT_SIZE),
            color: color.unwrap_or(Color::WHITE),
        },
    )
}

#[derive(Resource)]
pub struct FPSUpdateUITimer(Timer);
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct TpsText;

#[derive(Component)]
struct FrameTimeText;

#[derive(Component)]
struct PosText;

fn frame_time_update_system(
    time: Res<Time>,
    diag: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    mut timer: ResMut<FPSUpdateUITimer>,
) {
    // guard: timer hasn't finished, return early.
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut text in &mut query {
        let Some(fps) = diag.get(FrameTimeDiagnosticsPlugin::FPS).and_then(|fps| fps.smoothed()) else {
            return;
        };
        text.sections[1].value = format_value_f32(fps as f32, Some(2), false);
        //info!("text is this long {} and val is \"{}\" and the number is [{}]", text.sections[1].value.len(), val, fps);

        let Some(frame_time) = diag.get(FrameTimeDiagnosticsPlugin::FRAME_TIME).and_then(|frame_time| frame_time.smoothed()) else {
            return;
        };
        text.sections[3].value = format_value_f32(frame_time as f32, Some(2), false);
    }
}

fn tps_update_system(diag: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<TpsText>>) {
    for mut text in &mut query {
        text.sections[1].value = format!("idk");
    }
}

fn pos_update_system(
    camera_query: Query<(&Camera, &Transform, With<FlyCam>)>,
    mut text_query: Query<&mut Text, With<PosText>>,
) {
    for (_camera, transform, _) in &mut camera_query.into_iter() {
        for mut text in text_query.iter_mut() {
            text.sections[1].value = format!(
                "[{},{},{}]",
                format_value_f32(transform.translation.x, Some(2), true),
                format_value_f32(transform.translation.y, Some(2), true),
                format_value_f32(transform.translation.z, Some(2), true)
            );
        }
    }
}
