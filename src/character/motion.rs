use bevy::{
    input::ButtonInput, log::{info, warn}, math::Vec3, prelude::{Component, KeyCode, Query, Res, With, Without}, render::camera::{self, Camera}, time::Time, transform::components::Transform
};

use bevy_mod_picking::backend::prelude::PickSet::Input;
use bevy_xpbd_3d::components::{ExternalForce, ExternalImpulse, LinearVelocity};

use crate::{KeyBindings};

use super::{
    body::{self, Body},
    stance::Stance,
    Config, PlayerControl,
};

#[derive(Component)]
pub struct Motion {
    pub(crate) movement_vec: Vec3,
    pub(crate) sprinting: bool,
}

pub fn update_player_motion(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
    key_bindings: Res<KeyBindings>,
    camera_query: Query<&mut Transform, (With<Camera>, Without<PlayerControl>)>,
    mut query: Query<(
        &mut LinearVelocity,
        &mut Motion,
        &mut Stance,),
        With<PlayerControl>,
    >,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || query.is_empty()
        || query.iter().len() > 1
    {
        warn!("Player Motion System did not expected 1 camera(s) recieved {}, and 1 player(s) recieved {}", camera_query.iter().len(), query.iter().len());
    }

    for camera_transform in camera_query.iter() {
        for (mut linear_vel, mut motion, mut stance) in &mut query {
            // Perform the movement checks.
            let mut movement_vector: Vec3 = Vec3::ZERO.clone();

            // todo: this could be cleaned up by producing a Option if the key is down and unwrapping to the value or zero.
            let mut computed_speed: f32 = config.movement_speed;
            if keys.pressed(key_bindings.toggle_sprint) {
                computed_speed *= config.sprint_speed_factor;
                motion.sprinting = true;
            } else {
                motion.sprinting = false;
            }
            let speed_vector: Vec3 = Vec3::from([computed_speed, computed_speed, computed_speed]);

            if keys.pressed(key_bindings.move_forward) {
                movement_vector += camera_transform.forward().as_vec3();
            }
            if keys.pressed(key_bindings.move_backward) {
                movement_vector += camera_transform.back().as_vec3();
            }
            if keys.pressed(key_bindings.move_left) {
                movement_vector += camera_transform.left().as_vec3();
            }
            if keys.pressed(key_bindings.move_right) {
                movement_vector += camera_transform.right().as_vec3();
            }

            // apply the total movement vector.
            motion.movement_vec +=
                movement_vector.normalize_or_zero() * speed_vector * time.delta_seconds();
            // Appy decay to Linear Velocity on the X and Z directions and apply to the velocity.
            motion.movement_vec.x *= config.movement_decay;
            motion.movement_vec.z *= config.movement_decay;
            linear_vel.x = motion.movement_vec.x;
            linear_vel.z = motion.movement_vec.z;
        }
    }
}

pub fn apply_spring_force(
    config: &Res<Config>,
    linear_vel: &mut LinearVelocity,
    external_force: &mut ExternalForce,
    ray_length: f32,
) {
    // Find the diference between how close the capsule is to the surface beneath it.
    // Compute this value by subtracting the ray length from the set ride height
    // to find the diference in position.
    let spring_offset = f32::abs(ray_length) - config.ride_height;
    let spring_force =
        (spring_offset * config.ride_spring_strength) - (-linear_vel.y * config.ride_spring_damper);

    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
    external_force.clear();
    external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
}

pub fn apply_jump_force(
    config: &Res<Config>,
    stance: &mut Stance,
    external_impulse: &mut ExternalImpulse,
    ray_length: f32,
) {
    // Apply the stance cooldown now that we are jumping
    stance.lockout = config.stance_lockout;

    let half_jump_strength: f32 = config.jump_strength / 2.0;
    let jump_factor: f32 = compute_clamped_jump_force_factor(&config, ray_length);

    // make this value changable.
    let dynamic_jump_strength: f32 = half_jump_strength + (half_jump_strength * jump_factor);

    // todo: right now we are applying this jump force directly up, this needs to consider the original movement velocities.
    // maybe instead of half the strength getting added to the up we added it directionally only so you always jump x height but can
    // use more of the timing to aid in forward momentum.

    //remove any previous impulse on the object.
    external_impulse.clear();
    external_impulse.apply_impulse(Vec3::from((0.0, dynamic_jump_strength, 0.0)).into());

    info!(
        "\tJumped with {}/{} due to distance to ground, jump_factor {}, of ray length: {}",
        dynamic_jump_strength, config.jump_strength, jump_factor, ray_length
    );

    info!("\t ray_length {} ", ray_length);
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
fn compute_clamped_jump_force_factor(config: &Res<Config>, ray_length: f32) -> f32 {
    // Constants defined elsewhere in the code
    let full_standing_ray_length: f32 = config.ride_height;
    let half_standing_ray_length: f32 = config.ride_height - (config.capsule_height / 4.0);
    let standing_ray_length_range: f32 = full_standing_ray_length - half_standing_ray_length;

    // Ensure the input is within the specified range
    let clamped_ray_length = f32::clamp(ray_length, half_standing_ray_length, config.ride_height);

    // Apply the linear transformation
    // Step 1: Normalize clamped_ray_length to a value between 0.0 and 1.0
    let normalized_distance =
        (clamped_ray_length - half_standing_ray_length) / standing_ray_length_range;

    // Step 2: Subtract the normalized distance from CAPSULE_HEIGHT
    let result: f32 = config.capsule_height - normalized_distance;

    // Ensure the output is within the range [0.0, 1.0]
    f32::clamp(result, 0.0, 1.0)
}
