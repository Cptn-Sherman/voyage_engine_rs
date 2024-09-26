use avian3d::prelude::*;
use avian_pickup::{actor::AvianPickupActor, input::{AvianPickupAction, AvianPickupInput}};
use bevy::{log::info, prelude::*};
use body::Body;
use config::PlayerControlConfig;
use focus::Focus;
use motion::Motion;
use stance::{ActionStep, Stance, StanceType, ACTION_STEP_DELTA_DEFAULT};
use crate::{character::*, CameraThing, KeyBindings};

pub mod config;

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
    pickup_actor: AvianPickupActor,
}

pub fn spawn_player(
    player_config: Res<PlayerControlConfig>,
    mut commands: Commands,
) {
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
            body: Body { body_scale: 1.0 },
            motion: Motion {
                movement_vec: Vec3::from_array([0.0, 0.0, 0.0]),
                current_movement_speed: player_config.movement_speed,
                sprinting: false,
                moving: false,
                current_ride_height: player_config.ride_height,
                target_ride_height: player_config.ride_height,
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
            pickup_actor: AvianPickupActor::default(),
        },
        Player,
        
    ));
    info!("Spawned Player Actor");
}


pub fn handle_pickup_input(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    actor: Query<Entity, With<AvianPickupActor>>,
    mut avian_pickup_input_writer: EventWriter<AvianPickupInput>,
) {
    if keys.pressed(key_bindings.interact) {
        println!("Interact key pressed");
        avian_pickup_input_writer.send(AvianPickupInput {
            action: AvianPickupAction::Pull,
            actor: actor.iter().next().unwrap(),
        });
    }
}