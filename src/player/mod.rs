use avian3d::prelude::*;
use bevy::{log::info, prelude::*};

use crate::{camera::GameCamera, utils::grab_cursor};
use body::Body;
use config::PlayerControlConfig;
use focus::{camera_look_system, Focus};
use motion::{compute_motion, Motion};
use stance::{
    load_footstep_sfx, lock_rotation, play_footstep_sfx, tick_footstep, update_player_stance,
    ActionStep, FootstepEvent, Stance, StanceType, ACTION_STEP_DELTA_DEFAULT,
};
use states::{crouched::toggle_crouching, grounded::sprinting::toggle_sprint};

pub mod body;
pub mod config;
pub mod focus;
pub mod motion;
pub mod stance;
pub mod states;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerControlConfig::default()); // later we will load from some toml file
        app.add_systems(
            Startup,
            (
                load_footstep_sfx,
                spawn_player,
                attached_camera_system,
                grab_cursor,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                update_player_stance,
                toggle_crouching,
                toggle_sprint,
                compute_motion,
                lock_rotation,
                play_footstep_sfx,
                tick_footstep,
                camera_look_system,
            )
                .chain(),
        );
        app.add_event::<FootstepEvent>();
        info!("Initialized Player plugin");
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    linear_vel: LinearVelocity,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    mass: Mass,
    gravity_scale: GravityScale,
    collider: Collider,
    transform: Transform,
    global_transform: GlobalTransform,
    downward_ray: RayCaster,
    ray_hits: RayHits,
    body: Body,
    motion: Motion,
    focus: Focus,
    stance: Stance,
    action_step: ActionStep,
}

pub fn spawn_player(player_config: Res<PlayerControlConfig>, mut commands: Commands) {
    commands.spawn((
        PlayerBundle {
            linear_vel: LinearVelocity::ZERO,
            external_force: ExternalForce::new([0.0, 0.0, 0.0].into()),
            external_impulse: ExternalImpulse::new([0.0, 0.0, 0.0].into()),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::new(),
            mass: Mass(20.0),
            gravity_scale: GravityScale(1.0),
            collider: Collider::capsule(0.75, 0.5),
            transform: Transform::from_xyz(0.0, 16.0, 0.0),
            global_transform: GlobalTransform::default(),
            downward_ray: RayCaster::new(Vec3::ZERO, Dir3::NEG_Y),
            ray_hits: RayHits::default(),
            body: Body {
                current_body_height: 1.0,
            },
            motion: Motion {
                current_movement_speed: player_config.movement_speed,
                target_movement_speed: player_config.movement_speed,
                current_ride_height: player_config.ride_height,
                target_ride_height: player_config.ride_height,
                movement_vector: Vec3::from_array([0.0, 0.0, 0.0]),
                sprinting: false,
                moving: false,
            },
            focus: Focus {
                point_of_focus: Vec3::from_array([0.0, 0.0, 0.0]),
                face_direction: Vec3::from_array([0.0, 0.0, 0.0]),
                free_look: false,
            },
            stance: Stance {
                current: StanceType::Standing,
                crouched: false,
                lockout: 0.0,
            },
            action_step: ActionStep {
                dir: stance::FootstepDirection::Right,
                delta: ACTION_STEP_DELTA_DEFAULT,
                bumped: false,
            },
        },
        Player,
    ));
    info!("Spawned Player Actor");
}

fn attached_camera_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform), (With<Player>, Without<Camera>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, Option<&Parent>),
        (With<Camera3d>, With<GameCamera>, Without<Player>),
    >,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        warn!("The camera attach system did not recieve 1 player and 1 camera. Found {} cameras, and {} players", camera_query.iter().len(), player_query.iter().len());
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
