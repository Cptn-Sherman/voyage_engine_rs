use actions::{
    crouch::toggle_crouching,
    sprint::toggle_sprinting,
    step::{
        load_footstep_sfx, play_footstep_sfx, tick_footstep, ActionStep, FootstepDirection,
        FootstepEvent, ACTION_STEP_DELTA_DEFAULT,
    },
};
use avian3d::prelude::*;
use bevy::{log::info, prelude::*};

use crate::{
    camera::{smooth_camera, GameCamera},
    player::{
        focus::player_rotation_system,
        motion::{update_input_resource, Input},
    },
    user_interface::themes::{BORDER_COLOR, DEFAULT_DEBUG_FONT_PATH},
    utils::InterpolatedValue,
};
use body::Body;
use config::PlayerControlConfig;
use focus::{camera_look_system, Focus};
use motion::{
    compute_motion, update_debug_is_moving, update_debug_is_sprinting,
    update_debug_linear_velocity, update_debug_movement_speed_current,
    update_debug_movement_speed_target, update_debug_movement_vector_current,
    update_debug_movement_vector_decay, update_debug_movement_vector_target, update_debug_position,
    update_debug_rotation, Motion, MotionMovementIsMovingDebug, MotionMovementIsSprintingDebug,
    MotionMovementSpeedCurrentDebug, MotionMovementSpeedTargetDebug,
    MotionMovementVectorCurrentDebug, MotionMovementVectorDecayRateDebug,
    MotionMovementVectorTargetDebug, MotionPositionDebug, MotionRotationDebug, MotionVelocityDebug,
};
use stance::{lock_angular_velocity, update_player_stance, Stance, StanceType};

pub mod actions;
pub mod body;
pub mod config;
pub mod focus;
pub mod motion;
pub mod stance;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerControlConfig::default()); // later we will load from some toml file
        app.insert_resource(Input {
            movement: Vec3::from_array([0.0, 0.0, 0.0]),
            direction: Vec3::from_array([0.0, 0.0, 0.0]),
        });
        app.add_systems(
            Startup,
            (
                spawn_player,
                load_footstep_sfx,
                attached_camera_system,
                create_player_debug,
            )
                .chain(),
        );
        app.add_systems(
            FixedUpdate,
            (
                update_input_resource,
                update_player_stance,
                camera_look_system,
                player_rotation_system,
                compute_motion,
                smooth_camera,
                toggle_crouching,
                toggle_sprinting,
                lock_angular_velocity,
                play_footstep_sfx,
                tick_footstep,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                update_debug_movement_vector_decay,
                update_debug_movement_vector_current,
                update_debug_movement_vector_target,
                update_debug_movement_speed_current,
                update_debug_movement_speed_target,
                update_debug_linear_velocity,
                update_debug_is_sprinting,
                update_debug_is_moving,
                update_debug_rotation,
                update_debug_position,
            )
                .chain(),
        );
        app.add_event::<FootstepEvent>();
        info!("Initialized Player plugin");
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerColliderFlag;

#[derive(Bundle)]
pub struct PlayerColliderBundle {
    collider: Collider,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    linear_vel: LinearVelocity,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    downward_ray: RayCaster,
    ray_hits: RayHits,
    body: Body,
    motion: Motion,
    focus: Focus,
    stance: Stance,
    action_step: ActionStep,
    mass: Mass,
    locked_axes: LockedAxes,
    gravity_scale: GravityScale,
    transform: Transform,
    rigid_body: RigidBody,
}

pub fn spawn_player(
    player_config: Res<PlayerControlConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut collider = Collider::capsule(0.5, 1.0);
    collider.set_scale(Vec3::from([1.0, 1.0, 1.0]), 10);

    commands
        .spawn((
            PlayerBundle {
                linear_vel: LinearVelocity::ZERO,
                external_force: ExternalForce::new([0.0, 0.0, 0.0].into()),
                external_impulse: ExternalImpulse::new([0.0, 0.0, 0.0].into()),
                gravity_scale: GravityScale(1.0),
                transform: Transform::from_xyz(0.0, 16.0, 0.0),
                downward_ray: RayCaster::new(Vec3::ZERO, Dir3::NEG_Y),
                ray_hits: RayHits::default(),
                rigid_body: RigidBody::Dynamic,
                locked_axes: LockedAxes::new()
                    .lock_rotation_z()
                    .lock_rotation_x()
                    .lock_rotation_y(),
                mass: Mass(20.0),
                body: Body {
                    current_body_height: 1.0,
                },
                motion: Motion {
                    linear_velocity_interp: InterpolatedValue::new(
                        Vec3::from_array([0.0, 0.0, 0.0]),
                        16.0,
                    ),
                    movement_vector: InterpolatedValue::new(
                        Vec3::from_array([0.0, 0.0, 0.0]),
                        16.0,
                    ),
                    movement_speed: InterpolatedValue::new(
                        player_config.default_movement_speed,
                        4.0,
                    ),
                    sprinting: false,
                    moving: false,
                },
                stance: Stance {
                    ride_height: InterpolatedValue::new(player_config.ride_height, 6.0),
                    current: StanceType::Standing,
                    grounded: false,
                    crouched: false,
                    lockout: 0.0,
                },
                focus: Focus {
                    point_of_focus: Vec3::from_array([0.0, 0.0, 0.0]),
                    face_direction: Vec3::from_array([0.0, 0.0, 0.0]),
                    free_look: false,
                },
                action_step: ActionStep {
                    dir: FootstepDirection::Right,
                    delta: ACTION_STEP_DELTA_DEFAULT,
                    bumped: false,
                },
            },
            Mesh3d(meshes.add(Sphere::new(0.2).mesh().ico(8).unwrap())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 200.0 / 256.0, 0.0),
                ..default()
            })),
            TransformInterpolation,
            Player,
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerColliderBundle {
                    collider: collider.clone(),
                },
                PlayerColliderFlag,
            ));
        });
    info!("Spawned Player Actor");
}

fn attached_camera_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, Without<Camera>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, Option<&ChildOf>),
        (With<Camera3d>, With<GameCamera>, Without<Player>),
    >,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        warn!("The Camera attach system did not recieve 1 player and 1 camera. Found {} cameras, and {} players", camera_query.iter().len(), player_query.iter().len());
    }

    for (player_entity, _player_transform) in &mut player_query {
        for (camera_entity, mut camera_transform, camera_parent) in &mut camera_query {
            camera_transform.translation = Vec3::from_array([0.0, 1.0, 0.0]);
            if camera_parent.is_none() {
                commands
                    .entity(player_entity)
                    .add_children(&[camera_entity]);
                info!("Attached Camera to player character as child");
            } else {
                info!("Camera parent already exists, will not set player as parent! ");
            }
        }
    }
}

fn create_player_debug(mut commands: Commands, asset_server: Res<AssetServer>) {
    let default_font: Handle<Font> = asset_server.load(DEFAULT_DEBUG_FONT_PATH);
    let text_font = TextFont {
        font: default_font,
        font_size: 11.0,
        ..Default::default()
    };

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
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
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Text::new("pos: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionPositionDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("focus: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionRotationDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("vel: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionVelocityDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("Movement Vector | decay: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementVectorDecayRateDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("current: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementVectorCurrentDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("target: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementVectorTargetDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("moving: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementIsMovingDebug,
                            ));
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new(" | sprinting: "),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                            ));
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementIsSprintingDebug,
                            ));
                        });

                    parent
                        .spawn((
                            Text::new("current speed: "),
                            text_font.clone(),
                            TextColor(Color::WHITE),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementSpeedCurrentDebug,
                            ));
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new(" -> target: "),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                            ));
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextSpan::new("000"),
                                text_font.clone(),
                                TextColor(Color::WHITE),
                                MotionMovementSpeedTargetDebug,
                            ));
                        });
                });
        });

    info!("Created Player debug");
}
