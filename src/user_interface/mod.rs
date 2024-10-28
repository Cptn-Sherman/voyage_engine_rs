// basic setup for bevy plugin
use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_blur_regions::BlurRegion;

use crate::{
    utils::{format_percentage_f64, format_value_f32},
    CameraThing,
};

const DEFAULT_FONT_PATH: &str = "fonts/ashlander-pixel.ttf";
const DEFAULT_FONT_SIZE: f32 = 36.0;
const NO_PERCENTAGE: &str = "---.-%";

const ORANGE_TEXT_COLOR: Color = Color::hsv(0.34, 1.0, 0.5);
const YELLOW_GREEN_TEXT_COLOR: Color = Color::hsv(0.9, 0.69, 0.58);
const RED_TEXT_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const GOLD_TEXT_COLOR: Color = Color::srgb(1.0 , 0.72, 0.0);
const BORDER_COLOR: Color = Color::srgb(0.8 , 0.8, 0.8);
pub struct DebugInterfacePlugin;

impl Plugin for DebugInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ));
        app.add_systems(Startup, create_debug_interface);
        app.add_systems(
            Update,
            (
                frame_time_update_system,
                tps_debug_update_system,
                pos_debug_update_system,
                gpu_info_update_system,
                cpu_info_update_system,
            ),
        );
        app.insert_resource(FPSUpdateUITimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
    }
}

pub fn create_debug_interface(mut cmd: Commands, asset_server: Res<AssetServer>) {
    // setup the default font
    let default_font = asset_server.load(DEFAULT_FONT_PATH);

    // Spawn in the crosshair
    let cursor_size: f32 = 4.0;
    let cursor_color: BackgroundColor = BackgroundColor(Color::WHITE);
    let crosshair_texture_handle = asset_server.load("textures/white_square_crosshair.png");

    // Center Look UI
    cmd.spawn(NodeBundle {
        
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
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
    })
    .with_children(|parent| {
        parent
            .spawn(( BlurRegion, NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(3.0),
                    top: Val::Px(24.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.85)),
                border_color: BorderColor(BORDER_COLOR),
                ..Default::default()
            }))
            .with_children(|parent| {
                parent.spawn((TextBundle::from_sections([gen_text_section(
                    Some("Box".to_string()),
                    None,
                    Some(Color::WHITE),
                    default_font.clone(),
                )]),));
                parent.spawn((TextBundle::from_sections([gen_text_section(
                    Some("E: Take".to_string()),
                    None,
                    Some(YELLOW_GREEN_TEXT_COLOR),
                    default_font.clone(),
                )]),));
            });
    });

    // Engine and System Information

    // System State
    cmd.spawn( NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Absolute,
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent
            .spawn(( BlurRegion, NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.75)),
                border_color: BorderColor(BORDER_COLOR),
                ..Default::default()
            }))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("fps: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some("0".to_string()),
                            None,
                            Some(YELLOW_GREEN_TEXT_COLOR),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(" | ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some("0".to_string()),
                            None,
                            Some(YELLOW_GREEN_TEXT_COLOR),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(" ms/frame".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                    ]),
                    FpsText,
                ));
                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("tps: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(NO_PERCENTAGE.to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                    ]),
                    TpsText,
                ));
                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("gpu: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(NO_PERCENTAGE.to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(" | mem: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(NO_PERCENTAGE.to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                    ]),
                    GpuText,
                ));

                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("cpu: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(NO_PERCENTAGE.to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(" | mem: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        gen_text_section(
                            Some(NO_PERCENTAGE.to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                    ]),
                    CpuText,
                ));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("entity_count: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("---".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("hunk_count:   ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some(" 32".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("sys_time:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("3:35pm".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("last_save: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("3:35:36pm".to_string()),
                        None,
                        Some(GOLD_TEXT_COLOR),
                        default_font.clone(),
                    ),
                ]),));
            });
    })
    .with_children(|parent| {
        parent
            .spawn(( BlurRegion, NodeBundle {
                style: Style {
                    display: Display::Flex,
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::FlexStart,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    margin: UiRect::all(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.75)),
                border_color: BorderColor(BORDER_COLOR),
                ..Default::default()
            }))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("pos: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        TextSection::from_style(TextStyle {
                            font: asset_server.load(DEFAULT_FONT_PATH),
                            font_size: DEFAULT_FONT_SIZE,
                            color: GOLD_TEXT_COLOR,
                        }),
                    ]),
                    PosText,
                ));

                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("chunk: ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        TextSection::from_style(TextStyle {
                            font: asset_server.load(DEFAULT_FONT_PATH),
                            font_size: DEFAULT_FONT_SIZE,
                            color: Color::WHITE,
                        }),
                    ]),
                    PosText,
                ));

                // todo: spawn a horizontal line using parent.spawn to seperate the position from the chunk and hunk position.
                parent.spawn((
                    TextBundle::from_sections([
                        gen_text_section(
                            Some("hunk:  ".to_string()),
                            None,
                            Some(Color::WHITE),
                            default_font.clone(),
                        ),
                        TextSection::from_style(TextStyle {
                            font: asset_server.load(DEFAULT_FONT_PATH),
                            font_size: DEFAULT_FONT_SIZE,
                            color: Color::WHITE,
                        }),
                    ]),
                    PosText,
                ));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("nation:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("America".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));
                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("country:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("America".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));
                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("county:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("America".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));
                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("location:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("America".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                // note: the string for the biome could be quite long, so it might be better to have it on its own line.
                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("biome: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("liminal".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("weather: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("clear".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some(" | ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("temp: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("20 C".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                // ? not sure if global time is needed, but it could be useful for debugging?
                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("global_time: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("1:35:58pm".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("local_time:  ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("3:35:58pm".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("date: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("Oct 07 2023".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some(" | ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("fall".to_string()),
                        None,
                        Some(ORANGE_TEXT_COLOR),
                        default_font.clone(),
                    ),
                ]),));

                parent.spawn((TextBundle::from_sections([
                    gen_text_section(
                        Some("tod: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("evening".to_string()),
                        None,
                        Some(ORANGE_TEXT_COLOR),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some(" | ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("UTC: ".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                    gen_text_section(
                        Some("+02:00".to_string()),
                        None,
                        Some(Color::WHITE),
                        default_font.clone(),
                    ),
                ]),));
            });
    });
}

fn gen_text_section(
    value: Option<String>,
    size: Option<f32>,
    color: Option<Color>,
    font: Handle<Font>,
) -> TextSection {
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
struct CpuText;

#[derive(Component)]
struct GpuText;

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
        let Some(fps) = diag
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        else {
            return;
        };
        text.sections[1].value = format_value_f32(fps as f32, Some(2), false);

        let Some(frame_time) = diag
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|frame_time| frame_time.smoothed())
        else {
            return;
        };
        text.sections[3].value = format_value_f32(frame_time as f32, Some(2), false);
    }
}

fn tps_debug_update_system(
    diag: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<TpsText>>,
) {
    for mut text in &mut query {
        text.sections[1].value = format!("no_impl");
        text.sections[1].style.color = ORANGE_TEXT_COLOR;
    }
}

//
fn gpu_info_update_system(diag: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<GpuText>>) {
    for mut text in &mut query {
        //if let Some(gpu) = diag.get(SystemInformationDiagnosticsPlugin::GPU_USAGE).and_then(|gpu| gpu.smoothed()) {
        //    text.sections[1].value = format_value_f32(gpu as f32, Some(2), false);
        //    text.sections[1].style.color = Color::WHITE;
        //} else {
        text.sections[1].value =
            format_percentage_f64(Some(22.2)).unwrap_or(NO_PERCENTAGE.to_string());
        text.sections[1].style.color = ORANGE_TEXT_COLOR;
        text.sections[3].value = "no_impl".to_string();
        text.sections[3].style.color = ORANGE_TEXT_COLOR;
        //}
        // todo: there is current no plugin to get this information about the gpu, when it is available this will be updated. So this will always show no_impl.
        if let Some(mem) = diag
            .get(&SystemInformationDiagnosticsPlugin::MEM_USAGE)
            .and_then(|mem| mem.smoothed())
        {
            text.sections[3].value = format_value_f32(mem as f32, Some(2), false);
            text.sections[3].style.color = Color::WHITE;
        } else {
            text.sections[3].value = "no_impl".to_string();
            text.sections[3].style.color = RED_TEXT_COLOR;
        }
    }
}

fn cpu_info_update_system(diag: Res<DiagnosticsStore>, mut query: Query<&mut Text, With<CpuText>>) {
    for mut text in &mut query {
        let cpu = diag
            .get(&SystemInformationDiagnosticsPlugin::CPU_USAGE)
            .and_then(|cpu| cpu.smoothed());
        // BUG: there is a bug in the plugin, so it never returns a value, when its fixed this will start working.
        text.sections[1].value = format_percentage_f64(cpu).unwrap_or(NO_PERCENTAGE.to_string());
        text.sections[1].style.color = Color::hsv(0.51, 1.0, 0.5); // This color may be wrong...

        if let Some(mem) = diag
            .get(&SystemInformationDiagnosticsPlugin::MEM_USAGE)
            .and_then(|mem| mem.smoothed())
        {
            text.sections[3].value = format_value_f32(mem as f32, Some(2), false);
            text.sections[3].style.color = Color::WHITE;
        } else {
            text.sections[3].value = "no_data".to_string();
            text.sections[3].style.color = Color::srgb(1.0, 0.0, 0.0);
        }
    }
}

fn pos_debug_update_system(
    camera_query: Query<&Transform, With<CameraThing>>,
    mut query: Query<&mut Text, With<PosText>>,
) {
    for transform in &mut camera_query.into_iter() {
        for mut text in query.iter_mut() {
            text.sections[1].value = format!(
                "[{},{},{}]",
                format_value_f32(transform.translation.x, Some(2), true),
                format_value_f32(transform.translation.y, Some(2), true),
                format_value_f32(transform.translation.z, Some(2), true)
            );
        }
    }
}
