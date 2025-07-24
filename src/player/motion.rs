use bevy::{
    ecs::{component::Component, entity::Entity},
    gizmos::config,
    input::{keyboard::KeyCode, ButtonInput},
    log::{info, warn},
    math::{Vec3, Vec3Swizzles},
    prelude::{Query, Res, With},
    time::Time,
    transform::components::Transform,
};

use avian3d::prelude::*;

use crate::{
    input::Input,
    player::{
        body::{self, Body},
        stance::{compute_ray_length, StandingSpringForce},
    },
    ternary,
    utils::{exp_decay, format_value_vec3, InterpolatedValue},
};

use super::{
    stance::{Stance, StanceType},
    Player, PlayerControlConfig,
};

#[derive(Component)]
pub struct Motion {
    pub linear_velocity_interp: InterpolatedValue<Vec3>,
    pub movement_vector: InterpolatedValue<Vec3>,
    pub movement_speed: InterpolatedValue<f32>,
    pub sprinting: bool,
    pub moving: bool,
}

pub fn compute_motion(
    mut player_query: Query<
        (
            Entity,
            &mut StandingSpringForce,
            &mut LinearVelocity,
            &mut ExternalImpulse,
            &mut Transform,
            &mut Motion,
            &mut Stance,
            &Body,
            &RayHits,
        ),
        With<Player>,
    >,
    player_config: Res<PlayerControlConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    input: Res<Input>,
    time: Res<Time>,
) {
    if player_query.is_empty() || player_query.iter().len() > 1 {
        warn!(
            "Player Motion System expected 1 player(s), recieved {}. Expect Instablity!",
            player_query.iter().len()
        );
        return;
    }

    let (
        entity,
        mut standing_spring,
        mut linear_vel,
        mut external_impulse,
        player_transform,
        mut motion,
        mut stance,
        body,
        ray_hits,
    ) = player_query.single_mut().expect("We do some errors");

    // * - COMPUTE CURRENT MOVEMENT SPEED AND LERP -

    if motion.sprinting == true {
        if stance.crouched == true {
            motion.movement_speed.target =
                player_config.default_movement_speed * 0.5 * player_config.sprint_speed_factor;
        } else {
            motion.movement_speed.target =
                player_config.default_movement_speed * player_config.sprint_speed_factor;
        }
    } else {
        if stance.crouched == false {
            motion.movement_speed.target = player_config.default_movement_speed;
        } else {
            motion.movement_speed.target = player_config.default_movement_speed * 0.5;
        }
    }

    // Apply lineaer interpolation to move the speed transition.
    motion.movement_speed.current = exp_decay(
        motion.movement_speed.current,
        motion.movement_speed.target,
        motion.movement_speed.decay,
        time.delta_secs(),
    );

    // info!(
    //     "Movement Speed current: {}, target: {}",
    //     format_value_f32(motion.current_movement_speed, Some(4), true), format_value_f32(motion.target_movement_speed, Some(4), true)
    // );

    // * UPDATE MOVEMENT_VECTOR AND LERP

    let movement_scale: f32 = ternary!(
        stance.current != StanceType::Standing && stance.current != StanceType::Landing,
        0.5,
        1.0
    );

    let mut movement_vector: Vec3 = Vec3::ZERO.clone();
    // Apply the input_vector to the player to update the movement_vector.
    movement_vector += player_transform.forward().as_vec3() * input.movement_raw.z;
    movement_vector += player_transform.right().as_vec3() * input.movement_raw.x;

    // Update the target movement vector to be the normalized movement vector.
    motion.movement_vector.target = movement_vector.normalize_or_zero();

    // Lerp the current movement vector towards the target movement vector
    // updating the decay rate based on movement scale (based on being grounded or airborne)
    motion.movement_vector.current = exp_decay::<Vec3>(
        motion.movement_vector.current,
        motion.movement_vector.target,
        motion.movement_vector.decay * movement_scale,
        time.delta_secs(),
    );

    // info!(
    //     "Current Movement Vector: {}",
    //     format_value_vec3(motion.movement_vector.current, Some(4), true),
    // );

    // * APPLY MOVEMENT_VECTOR TO PLAYER TRANSFORM LINEAR VELOCITY

    // We don't need to lerp here just setting the real value to as we already lerp the current_movement_vector and current_movement_speed.
    if stance.current == StanceType::Standing {
        motion.linear_velocity_interp.target.x =
            motion.movement_vector.current.x * motion.movement_speed.current;
        motion.linear_velocity_interp.target.z =
            motion.movement_vector.current.z * motion.movement_speed.current;
    } else {
        let pi: f32 = 3.1459;
        let scale: f32 = 0.0;
        let offset: f32 = 0.1;

        let dot: f32 = motion
            .linear_velocity_interp
            .current
            .dot(motion.movement_vector.current);
        let clamped_movement_speed: f32 = motion.movement_speed.current.clamp(0.0, 5.0);
        let air_time_scale: f32 =
            ((1f32 - f32::cos(0.5 * pi * dot - 0.5 * pi)) / 2.0 - scale) + offset;
        let clamped_air_time_scaled_movement_speed: f32 =
            clamped_movement_speed * movement_scale * air_time_scale;
        // info!(
        //     "final movement speed: {}",
        //     clamped_air_time_scaled_movement_speed
        // );

        motion.linear_velocity_interp.target.x += motion.movement_vector.current.x
            * clamped_air_time_scaled_movement_speed
            * time.delta_secs();
        motion.linear_velocity_interp.target.z += motion.movement_vector.current.z
            * clamped_air_time_scaled_movement_speed
            * time.delta_secs();
    }

    motion.linear_velocity_interp.current = exp_decay::<Vec3>(
        motion.linear_velocity_interp.current,
        motion.linear_velocity_interp.target,
        motion.linear_velocity_interp.decay,
        time.delta_secs(),
    );

    linear_vel.x = motion.linear_velocity_interp.current.x;
    // linear_vel.y = motion.linear_velocity_interp.current.y;
    linear_vel.z = motion.linear_velocity_interp.current.z;

    info!(
        "Interpolated Linear Velocity: current {}",
        format_value_vec3(motion.linear_velocity_interp.current, Some(3), true)
    );
    info!(
        "Interpolated Linear Velocity: target {}",
        format_value_vec3(motion.linear_velocity_interp.target, Some(3), true)
    );
    info!(
        "Linear Velocity: {}",
        format_value_vec3(linear_vel.0, Some(4), true),
    );

    // * -   -

    // ! BUG:

    if stance.current == StanceType::Standing
        && keys.pressed(KeyCode::Space)
        && stance.lockout <= 0.0
    {
        let ray_length: f32 = compute_ray_length(entity, ray_hits);
        stance.current = StanceType::Airborne;
        stance.lockout = player_config.stance_lockout;
        linear_vel.y = 0.0;
        info!("APPLYING JUMP FORCE");
        apply_jump_force(
            &player_config,
            &mut external_impulse,
            ray_length,
            &mut stance,
            &mut standing_spring,
            &motion,
            &body,
        );
    }

        info!("External Impulse: {}", format_value_vec3(external_impulse.xyz(), Some(3), true));

    // * Detected and apply MOVING flag.
    // set the motion.moving when the magnituted of the movement_vector is greater than some arbitrary small threshold.
    motion.moving = motion.movement_vector.current.length() >= 0.01;
}

pub fn apply_jump_force(
    player_config: &Res<PlayerControlConfig>,
    external_impulse: &mut ExternalImpulse,
    ray_length: f32,
    stance: &mut Stance,
    standing_spring: &mut StandingSpringForce,
    motion: &Motion,
    body: &Body,
) {
    // Apply the stance cooldown now that we are jumping.
    stance.lockout = player_config.stance_lockout;

    let half_jump_strength: f32 = player_config.jump_strength / 2.0;
    let clamped_jump_force: f32 =
        compute_clamped_jump_force_factor(&body, &standing_spring, ray_length);

    // todo: make this value changable.
    let dynamic_jump_strength: f32 = half_jump_strength + (half_jump_strength * clamped_jump_force);

    // todo: right now we are applying this jump force directly up, this needs to consider the original movement velocities.
    // maybe instead of half the strength getting added to the up we added it directionally only so you always jump x height but can
    // use more of the timing to aid in forward momentum.

    // remove any previous impulse on the object.
    external_impulse.clear();
    info!("Cleared impulses");

    // find the movement vector in the x and z direction.
    let normalized_midpoint_movement_vector: Vec3 = Vec3 {
        x: motion.movement_vector.current.x,
        y: 1.0,
        z: motion.movement_vector.current.z,
    };

    // info!(
    //     "normalized_midpoint_movement_vector: {}",
    //     format_value_vec3(normalized_midpoint_movement_vector, Some(3), true)
    // );

    let impulse_vec: Vec3 = Vec3::from((
        normalized_midpoint_movement_vector.x * dynamic_jump_strength,
        normalized_midpoint_movement_vector.y * dynamic_jump_strength,
        normalized_midpoint_movement_vector.z * dynamic_jump_strength,
    ));

    // info!("impulse_vec: {}", format_value_vec3(impulse_vec, Some(3), true));

    // apply the jump force.
    external_impulse.apply_impulse(impulse_vec.into());
    
    info!(
        "\tJumped with {}/{} due to distance to ground, jump_factor {}, of ray length: {}",
        dynamic_jump_strength, player_config.jump_strength, clamped_jump_force, ray_length
    );
}

fn compute_clamped_jump_force_factor(
    body: &Body,
    standing_spring: &StandingSpringForce,
    ray_length: f32,
) -> f32 {
    // Constants defined elsewhere in the code
    let full_standing_ray_length: f32 = standing_spring.length.current;
    let half_standing_ray_length: f32 =
        standing_spring.length.current - (body.current_body_height / 4.0);
    // This value represents the range of acceptable ray lengths for the player.
    let standing_ray_length_range: f32 = full_standing_ray_length - half_standing_ray_length;

    // Ensure the input is within the specified range
    let clamped_ray_length = f32::clamp(
        ray_length,
        half_standing_ray_length,
        standing_spring.length.current,
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
    let spring_offset: f32 = f32::abs(ray_length) - ride_height;
    let spring_force: f32 =
        (spring_offset * config.ride_spring_strength) - (-linear_vel.y * config.ride_spring_damper);

    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
    external_force.clear();
    external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));

    
}
