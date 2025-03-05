use bevy::{
    ecs::{component::Component, entity::Entity},
    input::{
        gamepad::{Gamepad, GamepadAxis},
        ButtonInput,
    },
    log::{info, warn},
    math::{EulerRot, Quat, Vec3},
    prelude::{Camera3d, KeyCode, Query, Res, With, Without},
    text::TextSpan,
    time::Time,
    transform::components::Transform,
};

use avian3d::prelude::*;

use crate::{
    ternary,
    utils::{exp_decay, exp_vec3_decay, format_value_f32, format_value_quat, format_value_vec3},
    Bindings,
};

use super::{
    body::Body,
    stance::{Stance, StanceType},
    Player, PlayerControlConfig,
};

const ANALOGE_STICK_DEADZONE: f32 = 0.1;

#[derive(Component)]
pub struct Motion {
    pub(crate) current_movement_vector: Vec3,
    pub(crate) target_movement_vector: Vec3,
    pub(crate) current_movement_speed: f32,
    pub(crate) target_movement_speed: f32,
    pub(crate) current_lean: Vec3,
    pub(crate) target_lean: Vec3,
    pub(crate) sprinting: bool,
    pub(crate) moving: bool,
}


pub fn compute_motion(
    mut player_query: Query<
        (&mut LinearVelocity, &mut Transform, &mut Motion, &Stance),
        With<Player>,
    >,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    player_config: Res<PlayerControlConfig>,
    gamepads: Query<(Entity, &Gamepad)>,
    key_bindings: Res<Bindings>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        warn!("Player Motion System did not expected 1 camera(s) recieved {}, and 1 player(s) recieved {}. Expect Instablity!", camera_query.iter().len(), player_query.iter().len());
        return;
    }

    let mut camera_transform = camera_query.single_mut();
    let (mut linear_vel, player_transform, mut motion, stance) = player_query.single_mut();

    let movement_scale = ternary!(
        stance.current != StanceType::Standing && stance.current != StanceType::Landing,
        0.15,
        1.0
    );

    // this is the raw input vector
    let mut input_vector: Vec3 = Vec3::ZERO.clone();
    let mut movement_vector: Vec3 = Vec3::ZERO.clone();

    if keys.pressed(key_bindings.move_forward) {
        input_vector.z = 1.0;
        movement_vector += player_transform.forward().as_vec3();
    }
    if keys.pressed(key_bindings.move_backward) {
        input_vector.z = -1.0;
        movement_vector += player_transform.back().as_vec3();
    }
    if keys.pressed(key_bindings.move_left) {
        input_vector.x = 1.0;
        movement_vector += player_transform.left().as_vec3();
    }
    if keys.pressed(key_bindings.move_right) {
        input_vector.x = -1.0;
        movement_vector += player_transform.right().as_vec3();
    }

    if let Ok((_entity, gamepad)) = gamepads.get_single() {
        let left_stick_x: f32 = gamepad.get(GamepadAxis::LeftStickX).unwrap_or_default();
        let left_stick_y: f32 = gamepad.get(GamepadAxis::LeftStickY).unwrap_or_default();

        if left_stick_x.abs() > ANALOGE_STICK_DEADZONE {
            movement_vector += player_transform.right().as_vec3() * left_stick_x;
            input_vector.x = left_stick_x;
        }

        if left_stick_y.abs() > ANALOGE_STICK_DEADZONE {
            movement_vector += player_transform.forward().as_vec3() * left_stick_y;
            input_vector.y = left_stick_y;
        }
    }

    let current_movement_vector_decay: f32 = 16.0;

    motion.current_movement_vector = exp_vec3_decay(
        motion.current_movement_vector,
        motion.target_movement_vector,
        current_movement_vector_decay * movement_scale, // TODO: this is not functioning right
        time.delta_secs(),
    );

    // set the motion.moving when the magnituted of the movement_vector is greater than some arbitrary threshold.
    motion.moving = motion.current_movement_vector.length() >= 0.01;

    let movement_scale: f32 = f32::clamp(movement_vector.length(), 0.0, 1.0);

    if motion.sprinting == true {
        if stance.crouched == true {
            motion.target_movement_speed = player_config.movement_speed
                * 0.5
                * player_config.sprint_speed_factor
                * movement_scale;
        } else {
            motion.target_movement_speed =
                player_config.movement_speed * player_config.sprint_speed_factor * movement_scale;
        }
    } else {
        if stance.crouched == false {
            motion.target_movement_speed = player_config.movement_speed * movement_scale;
        } else {
            motion.target_movement_speed = player_config.movement_speed * 0.5 * movement_scale;
        }
    }

    let movement_speed_decay: f32 = 4.0;

    motion.current_movement_speed = exp_decay(
        motion.current_movement_speed,
        motion.target_movement_speed,
        movement_speed_decay,
        time.delta_secs(),
    );

    // Update the Curent Lean

    let rotation_amount: f32 = 2.0;
    let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::default());
    //let pitch = input_vector.y * rotation_amount.to_radians();
    let roll = input_vector.x * rotation_amount.to_radians();
    motion.target_lean = Vec3::from_array([yaw, pitch, roll]);

    let current_lean_decay: f32 = 8.0;

    motion.current_lean = exp_vec3_decay(
        motion.current_lean,
        motion.target_lean,
        current_lean_decay,
        time.delta_secs(),
    );

    motion.target_movement_vector = movement_vector.normalize_or_zero();

    // Update the player lean
    camera_transform.rotation = Quat::from_euler(
        EulerRot::default(),
        yaw, // we dont change the yaw.
        pitch,
        motion.current_lean.z,
    );

    // we don't need to lerp here just setting the real value to as we already lerp the current_movement_vector and current_movement_speed.
    linear_vel.x = motion.current_movement_vector.x * motion.current_movement_speed;
    linear_vel.z = motion.current_movement_vector.z * motion.current_movement_speed;

    // info!(
    //     "Movement Speed current: {}, target: {}",
    //     format_value_f32(motion.current_movement_speed, Some(4), true), format_value_f32(motion.target_movement_speed, Some(4), true)
    // );
    // info!(
    //     "Current Movement Vector: [{}, {}, {}]",
    //     format_value_f32(motion.current_movement_vector.x, Some(4), true),
    //     format_value_f32(motion.current_movement_vector.y, Some(4), true),
    //     format_value_f32(motion.current_movement_vector.z, Some(4), true)
    // );
    // info!(
    //     "Linear Velocity: [{}, {}, {}]",
    //     format_value_f32(linear_vel.x, Some(4), true),
    //     format_value_f32(linear_vel.y, Some(4), true),
    //     format_value_f32(linear_vel.z, Some(4), true)
    // );
}

pub fn apply_spring_force(
    config: &Res<PlayerControlConfig>,
    linear_vel: &mut LinearVelocity,
    external_force: &mut ExternalForce,
    ray_length: f32,
    ride_height: f32,
) {
    // Find the diference between how close the capsule is to the surface beneath it.
    // Compute this value by subtracting the ray length from the set ride height
    // to find the diference in position.
    let spring_offset = f32::abs(ray_length) - ride_height;
    let spring_force =
        (spring_offset * config.ride_spring_strength) - (-linear_vel.y * config.ride_spring_damper);
    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
    external_force.clear();
    external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
}

pub fn apply_jump_force(
    player_config: &Res<PlayerControlConfig>,
    stance: &mut Stance,
    external_impulse: &mut ExternalImpulse,
    linear_vel: &mut LinearVelocity,
    ray_length: f32,
    body: &Body,
) {
    // Apply the stance cooldown now that we are jumping.
    stance.lockout = player_config.stance_lockout;

    let half_jump_strength: f32 = player_config.jump_strength / 2.0;
    let jump_factor: f32 = compute_clamped_jump_force_factor(&body, &stance, ray_length);

    // make this value changable.
    let dynamic_jump_strength: f32 = half_jump_strength + (half_jump_strength * jump_factor);

    // todo: right now we are applying this jump force directly up, this needs to consider the original movement velocities.
    // maybe instead of half the strength getting added to the up we added it directionally only so you always jump x height but can
    // use more of the timing to aid in forward momentum.

    // remove any previous impulse on the object.
    external_impulse.clear();
    // find the movement vector in the x and z direction.
    let scaled_movement_vector: Vec3 =
        Vec3::from((linear_vel.x, 0.0, linear_vel.z)).normalize_or_zero();

    // apply the jump force.
    external_impulse.apply_impulse(
        Vec3::from((
            scaled_movement_vector.x,
            dynamic_jump_strength,
            scaled_movement_vector.z,
        ))
        .into(),
    );

    info!(
        "\tJumped with {}/{} due to distance to ground, jump_factor {}, of ray length: {}",
        dynamic_jump_strength, player_config.jump_strength, jump_factor, ray_length
    );
}

/// Computes a clamped jump force factor based on the provided ray length.
///
/// # Arguments
///
/// * `ray_length` - The length of the ray used in the computation.
///
/// # Returns
///
/// The clamped jump force factor within the range [0.0, 1.0].
///
/// # Examples
///
/// ```
/// let ray_length = 3.0;
/// let jump_force_factor = compute_clamped_jump_force_factor(ray_length);
/// println!("Jump Force Factor: {}", jump_force_factor);
/// ```
fn compute_clamped_jump_force_factor(body: &Body, stance: &Stance, ray_length: f32) -> f32 {
    // Constants defined elsewhere in the code
    let full_standing_ray_length: f32 = stance.current_ride_height;
    let half_standing_ray_length: f32 =
        stance.current_ride_height - (body.current_body_height / 4.0);
    // This value represents the range of acceptable ray lengths for the player.
    let standing_ray_length_range: f32 = full_standing_ray_length - half_standing_ray_length;

    // Ensure the input is within the specified range
    let clamped_ray_length = f32::clamp(
        ray_length,
        half_standing_ray_length,
        stance.current_ride_height,
    );

    // Apply the linear transformation
    // Step 1: Normalize clamped_ray_length to a value between 0.0 and 1.0.
    let normalized_distance =
        (clamped_ray_length - half_standing_ray_length) / standing_ray_length_range;

    // Step 2: Subtract the normalized distance from CAPSULE_HEIGHT.
    let result: f32 = body.current_body_height - normalized_distance;

    // Ensure the output is within the range [0.0, 1.0].
    f32::clamp(result, 0.0, 1.0)
}


#[derive(Component)]
pub struct MotionPositionDebug;

pub fn update_debug_position(
    player_query: Query<&Transform, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionPositionDebug>>,
) {
    let mut text = query.single_mut();
    let player_transform = player_query.single();
    text.0 = format_value_vec3(player_transform.translation, Some(4), false);
}

#[derive(Component)]
pub struct MotionRotationDebug;

pub fn update_debug_rotation(
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    mut query: Query<&mut TextSpan, With<MotionRotationDebug>>,
) {
    let camera_transform = camera_query.single();
    let player_transform = player_query.single();
    let (player_yaw, _player_pitch, _player_roll) = player_transform.rotation.to_euler(EulerRot::default());
    let (_camera_yaw,cmaera_pitch, camera_roll) = camera_transform.rotation.to_euler(EulerRot::default());
    let quat = Quat::from_euler(EulerRot::default(), player_yaw, cmaera_pitch, camera_roll);
    let mut text = query.single_mut();
    text.0 = format_value_quat(quat, Some(4), false, Some(EulerRot::default()));
}

#[derive(Component)]
pub struct MotionVelocityDebug;

pub fn update_debug_linear_velocity(
    player_query: Query<&mut LinearVelocity, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionVelocityDebug>>,
) {
    let mut text = query.single_mut();
    let player_linear_velocity = player_query.single();
    text.0 = format_value_vec3(player_linear_velocity.0, Some(4), false);
}

#[derive(Component)]
pub struct MotionMovementVectorCurrentDebug;

pub fn update_debug_movment_vector_current(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementVectorCurrentDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = format_value_vec3(player_motion.current_movement_vector, Some(4), false);
}

#[derive(Component)]
pub struct MotionMovementVectorTargetDebug;

pub fn update_debug_movment_vector_target(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementVectorTargetDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = format_value_vec3(player_motion.target_movement_vector, Some(4), false);
}

#[derive(Component)]
pub struct MotionMovementVectorDecayRateDebug;

pub fn update_debug_movment_vector_decay(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementVectorDecayRateDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    //text.0 = format_value_vec3(player_motion, Some(4), false);
}

#[derive(Component)]
pub struct MotionMovementIsMovingDebug;

pub fn update_debug_is_moving(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementIsMovingDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = player_motion.moving.to_string();
}

#[derive(Component)]
pub struct MotionMovementIsSprintingDebug;

pub fn update_debug_is_sprinting(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementIsSprintingDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = player_motion.sprinting.to_string();
}

#[derive(Component)]
pub struct MotionMovementSpeedCurrentDebug;

pub fn update_debug_movement_speed_current(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementSpeedCurrentDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = format_value_f32(player_motion.current_movement_speed, Some(4), false);
}

#[derive(Component)]
pub struct MotionMovementSpeedTargetDebug;

pub fn update_debug_movement_speed_target(
    player_query: Query<&Motion, With<Player>>,
    mut query: Query<&mut TextSpan, With<MotionMovementSpeedTargetDebug>>,
) {
    let mut text = query.single_mut();
    let player_motion = player_query.single();
    text.0 = format_value_f32(player_motion.target_movement_speed, Some(4), false);
}