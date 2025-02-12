pub mod config;
pub mod themes;

// basic setup for bevy plugin
use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use themes::{BORDER_COLOR, DEFAULT_FONT_PATH};

use crate::utils::format_value_f32;

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
                tick_timer_system,
                frame_time_update_system,
                ms_per_frame_system,
            )
                .chain(),
        );
        app.insert_resource(FPSUpdateUITimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
    }
}

pub fn create_debug_interface(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let default_font: Handle<Font> = asset_server.load(DEFAULT_FONT_PATH);
    let text_font = TextFont {
        font: default_font,
        font_size: 18.0,
        ..Default::default()
    };

    // // Spawn in the crosshair
    // let cursor_size: f32 = 4.0;
    // let cursor_color: BackgroundColor = BackgroundColor(Color::WHITE);
    // let crosshair_texture_handle = asset_server.load("textures/white_square_crosshair.png");

    // System State
    cmd.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::FlexEnd,
        position_type: PositionType::Absolute,
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            Node {
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
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.75)),
            BorderColor(BORDER_COLOR),
        )).with_children(|parent| {
            parent.spawn((Text::new("fps: "), text_font.clone(), TextColor(Color::WHITE))).with_children(|parent| {
                parent.spawn((TextSpan::new("000"), text_font.clone(), TextColor(Color::WHITE), FpsText)).with_children(|parent| {
                    parent.spawn((TextSpan::new(" | "), text_font.clone(), TextColor(Color::WHITE))).with_children(|parent| {
                        parent.spawn((TextSpan::new("00.0"), text_font.clone(), TextColor(Color::WHITE), MsText)).with_children(|parent| {
                            parent.spawn((TextSpan::new("  ms/frame"), text_font.clone(), TextColor(Color::WHITE)));
                        });
                    });
                });
            });
        });
    });
    

    // ? need to update this because the text is not in the right position in the hierarchy.
    // ? So it renders beneath the box is should be inside of. Additionally lets have the position on screen be adjustable like upper right, upper left, lower right, lower left and upper center and lower center.
}

#[derive(Resource)]
pub struct FPSUpdateUITimer(Timer);
#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct MsText;

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

fn tick_timer_system(time: Res<Time>, mut timer: ResMut<FPSUpdateUITimer>) {
    timer.0.tick(time.delta());
}

fn frame_time_update_system(
    timer: ResMut<FPSUpdateUITimer>,
    diag: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    // guard: timer hasn't finished, return early.
    if !timer.0.just_finished() {
        return;
    }

    for mut text in &mut query {
        let Some(fps) = diag
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        else {
            return;
        };
        text.0 = format_value_f32(fps as f32, Some(2), false);
    }
}

fn ms_per_frame_system(
    timer: ResMut<FPSUpdateUITimer>,
    diag: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<MsText>>,
) {
    // guard: timer hasn't finished, return early.
    if !timer.0.just_finished() {
        return;
    }

    for mut text in &mut query {
        let Some(frame_time) = diag
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(|frame_time| frame_time.smoothed())
        else {
            return;
        };
        text.0 = format_value_f32(frame_time as f32, Some(2), false);
    }
}

// setup the default font
// let default_font = asset_server.load(DEFAULT_FONT_PATH);
// // Spawn in the crosshair
// let cursor_size: f32 = 4.0;
// let cursor_color: BackgroundColor = BackgroundColor(Color::WHITE);
// let crosshair_texture_handle = asset_server.load("textures/white_square_crosshair.png");

// // Center Look UI
// cmd.spawn(Node {
//         width: Val::Percent(100.0),
//         height: Val::Percent(100.0),
//         flex_direction: FlexDirection::Column,
//         justify_content: JustifyContent::Center,
//         align_items: AlignItems::Center,
//         position_type: PositionType::Absolute,
//         ..default()
// })
// .with_children(|parent| {
//     parent.spawn((
//         Node {
//             width: Val::Px(cursor_size),
//             height: Val::Px(cursor_size),
//             ..default()
//             // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
//             //background_color: cursor_color.into(),
//         },
//         Image::new(crosshair_texture_handle.into()),
//     ));
// })
// .with_children(|parent| {
//     parent
//         .spawn((
//             BlurRegion,
//             NodeBundle {
//                 style: Style {
//                     display: Display::Flex,
//                     justify_content: JustifyContent::SpaceAround,
//                     align_items: AlignItems::Center,
//                     flex_direction: FlexDirection::Column,
//                     row_gap: Val::Px(8.0),
//                     top: Val::Px(8.0),
//                     padding: UiRect::all(Val::Px(8.0)),
//                     border: UiRect::all(Val::Px(2.0)),
//                     ..Default::default()
//                 },
//                 background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.85)),
//                 border_color: BorderColor(BORDER_COLOR),
//                 ..Default::default()
//             },
//         ))
//         .with_children(|parent| {
//             parent.spawn((TextBundle::from_sections([gen_text_section(
//                 Some("Box".to_string()),
//                 None,
//                 Some(Color::WHITE),
//                 default_font.clone(),
//             )]),));
//             parent.spawn((TextBundle::from_sections([gen_text_section(
//                 Some("E: Take".to_string()),
//                 None,
//                 Some(YELLOW_GREEN_TEXT_COLOR),
//                 default_font.clone(),
//             )]),));
//         });
// });

//             parent.spawn((
//                 TextBundle::from_sections([
//                     gen_text_section(
//                         Some("gpu: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(NO_PERCENTAGE.to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(" | mem: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(NO_PERCENTAGE.to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                 ]),
//                 GpuText,
//             ));

//             parent.spawn((
//                 TextBundle::from_sections([
//                     gen_text_section(
//                         Some("cpu: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(NO_PERCENTAGE.to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(" | mem: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     gen_text_section(
//                         Some(NO_PERCENTAGE.to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                 ]),
//                 CpuText,
//             ));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("entity_count: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("---".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("hunk_count:   ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some(" 32".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("sys_time:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("3:35pm".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("last_save: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("3:35:36pm".to_string()),
//                     None,
//                     Some(GOLD_TEXT_COLOR),
//                     default_font.clone(),
//                 ),
//             ]),));
//         });
// })
// .with_children(|parent| {
//     parent
//         .spawn((
//             BlurRegion,
//             NodeBundle {
//                 style: Style {
//                     display: Display::Flex,
//                     justify_content: JustifyContent::SpaceAround,
//                     align_items: AlignItems::FlexStart,
//                     flex_direction: FlexDirection::Column,
//                     row_gap: Val::Px(2.0),
//                     margin: UiRect::all(Val::Px(5.0)),
//                     padding: UiRect::all(Val::Px(5.0)),
//                     border: UiRect::all(Val::Px(2.0)),
//                     ..Default::default()
//                 },
//                 background_color: BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.75)),
//                 border_color: BorderColor(BORDER_COLOR),
//                 ..Default::default()
//             },
//         ))
//         .with_children(|parent| {
//             parent.spawn((
//                 TextBundle::from_sections([
//                     gen_text_section(
//                         Some("pos: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     TextSection::from_style(TextStyle {
//                         font: asset_server.load(DEFAULT_FONT_PATH),
//                         font_size: DEFAULT_FONT_SIZE,
//                         color: GOLD_TEXT_COLOR,
//                     }),
//                 ]),
//                 PosText,
//             ));

//             parent.spawn((
//                 TextBundle::from_sections([
//                     gen_text_section(
//                         Some("chunk: ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     TextSection::from_style(TextStyle {
//                         font: asset_server.load(DEFAULT_FONT_PATH),
//                         font_size: DEFAULT_FONT_SIZE,
//                         color: Color::WHITE,
//                     }),
//                 ]),
//             ));

//             // todo: spawn a horizontal line using parent.spawn to seperate the position from the chunk and hunk position.
//             parent.spawn((
//                 TextBundle::from_sections([
//                     gen_text_section(
//                         Some("hunk:  ".to_string()),
//                         None,
//                         Some(Color::WHITE),
//                         default_font.clone(),
//                     ),
//                     TextSection::from_style(TextStyle {
//                         font: asset_server.load(DEFAULT_FONT_PATH),
//                         font_size: DEFAULT_FONT_SIZE,
//                         color: Color::WHITE,
//                     }),
//                 ]),
//             ));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("nation:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("America".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));
//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("country:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("America".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));
//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("county:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("America".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));
//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("location:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("America".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             // note: the string for the biome could be quite long, so it might be better to have it on its own line.
//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("biome: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("liminal".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("weather: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("clear".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some(" | ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("temp: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("20 C".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             // ? not sure if global time is needed, but it could be useful for debugging?
//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("global_time: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("1:35:58pm".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("local_time:  ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("3:35:58pm".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((TextBundle::from_sections([
//                 gen_text_section(
//                     Some("date: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("Oct 07 2023".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some(" | ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("fall".to_string()),
//                     None,
//                     Some(ORANGE_TEXT_COLOR),
//                     default_font.clone(),
//                 ),
//             ]),));

//             parent.spawn((Text::new([
//                 gen_text_section(
//                     Some("tod: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("evening".to_string()),
//                     None,
//                     Some(ORANGE_TEXT_COLOR),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some(" | ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("UTC: ".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//                 gen_text_section(
//                     Some("+02:00".to_string()),
//                     None,
//                     Some(Color::WHITE),
//                     default_font.clone(),
//                 ),
//             ]),));
//         });
// });
