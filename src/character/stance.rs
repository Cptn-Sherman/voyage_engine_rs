use bevy::{
    input::Input,
    log::info,
    math::Vec3,
    prelude::{Component, KeyCode, Query, Res},
    time::Time,
};
use bevy_xpbd_3d::{
    components::{ExternalForce, ExternalImpulse, GravityScale, LinearVelocity},
    prelude::{RayCaster, RayHits},
};

use crate::KeyBindings;

use super::{
    motion::{apply_jump_force, apply_spring_force},
    Config,
};

#[derive(Debug, PartialEq, Clone)]
// each of these stance types needs to have a movement speed calculation, a
pub enum StanceType {
    Airborne,
    Standing,
    Landing,
    Jumping,
    Crouching,
    Crawling,
    Prone,
    Sliding,
    Vaulting,
    Hanging,
    Climbing,
}

#[derive(Component)]
pub struct Stance {
    pub current: StanceType,
    pub lockout: f32,
}

pub fn update_player_stance(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    config: Res<Config>,
    mut query: Query<(
        &RayCaster,
        &RayHits,
        &mut LinearVelocity,
        &mut GravityScale,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut Stance,
    )>,
) {
    for (ray, hits, mut vel, mut gravity, mut external_force, mut external_impulse, mut stance) in
    &mut query
    {
        // We update stance_lockout.
        stance.lockout -= time.delta_seconds();
        stance.lockout = f32::clamp(stance.lockout, 0.0, 1.0);
        
        // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
        let mut ray_length: f32 = f32::INFINITY;
        if let Some(hit) = hits.iter_sorted().next() {
            ray_length = Vec3::length(ray.direction * hit.time_of_impact);
        }
        
        // Compute the next stance for the player.
        let next_stance: StanceType = determine_next_stance(&keys, &config, &stance, ray_length);
        let mut gravity_scale = 1.0;
        
        match next_stance {
            StanceType::Landing => {
                // Set the gravity scale to zero.
                gravity_scale = 0.0;
                apply_spring_force(&config, &mut external_force, ray_length, vel.y);
            }
            StanceType::Standing => {
                // Set the gravity scale to zero.
                gravity_scale = 0.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();

                apply_spring_force(&config, &mut external_force, ray_length, vel.y);
            }
            StanceType::Airborne => {
                gravity_scale = 1.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();
            }
            StanceType::Jumping => {
                // set the gravity scale to zero.
                gravity_scale = 1.0;

                // clear any persisting forces on the rigid body.
                external_force.clear();

                // check if the stance has changed.
                if stance.current != StanceType::Jumping {
                    vel.y = 0.0; // clear the jump velocity.
                    apply_jump_force(&config, &mut stance, &mut external_impulse, ray_length);
                }
            }
            StanceType::Crouching => todo!(),
            StanceType::Crawling => todo!(),
            StanceType::Prone => todo!(),
            StanceType::Sliding => todo!(),
            StanceType::Vaulting => todo!(),
            StanceType::Hanging => todo!(),
            StanceType::Climbing => todo!(),
        }

        // Update the gravity scale.
        gravity.0 = gravity_scale;

        // Update the current stance.
        stance.current = next_stance.clone();
    }
}

fn determine_next_stance(
    keys: &Res<Input<KeyCode>>,
    config: &Res<Config>,
    stance: &Stance,
    ray_length: f32,
) -> StanceType {
    let is_locked_out: bool = stance.lockout > 0.0;
    let previous_stance: StanceType = stance.current.clone();
    let mut next_stance: StanceType = stance.current.clone();
    // If your locked in you cannot change state.
    if !is_locked_out {
        if ray_length > config.downward_ray_length_max {
            next_stance = StanceType::Airborne;
        } else if previous_stance == StanceType::Standing
            && stance.lockout <= 0.0
            && keys.pressed(KeyCode::Space)
        {
            next_stance = StanceType::Jumping;
        } else if ray_length < config.ride_height {
            next_stance = StanceType::Standing;
        } else if previous_stance != StanceType::Standing
            && ray_length < config.downward_ray_length_max
        {
            next_stance = StanceType::Landing;
        } else if ray_length > config.downward_ray_length_max {
            next_stance = StanceType::Airborne;
        }
    }

    if next_stance != previous_stance {
        info!(
            "Detected Stance Change:{:#?} -> {:#?}",
            previous_stance, next_stance
        );
    }

    return next_stance;
}
