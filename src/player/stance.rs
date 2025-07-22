use super::Player;
use super::{
    actions::step::{FootstepDirection, FootstepEvent},
    motion::{ apply_spring_force},
};
use super::{body::Body, PlayerColliderFlag};
use crate::player::config::PlayerControlConfig;
use crate::player::motion::Motion;
use crate::utils::{exp_decay, InterpolatedValue};
use avian3d::prelude::*;
use bevy::{
    ecs::entity::Entity,
    input::{
        gamepad::{Gamepad, GamepadButton},
        ButtonInput,
    },
    log::{info, warn},
    math::Vec3,
    prelude::{Component, EventWriter, KeyCode, Query, Res, With},
    time::Time,
};

#[derive(Debug, PartialEq, Clone)]
// each of these stance types needs to have a movement speed calculation, a
pub enum StanceType {
    Airborne,
    Standing,
    Landing,
    Jumping,
}

#[derive(Component)]
pub struct Stance {
    pub current: StanceType,
    pub _grounded: bool,
    pub crouched: bool,
    pub lockout: f32,
}

// todo: I want to try making it faster to move "forward" and slower to move left, right or backwards. Maybe we construct a const movement speed scaler for each direction.
// pub fn update_player_stance(
//     time: Res<Time>,
//     keys: Res<ButtonInput<KeyCode>>,
//     config: Res<PlayerControlConfig>,
//     gamepad_query: Query<(Entity, &Gamepad)>,
//     mut query: Query<
//         (
//             &mut LinearVelocity,
//             &mut ExternalForce,
//             &mut ExternalImpulse,
//             &mut GravityScale,
//             &mut Stance,
//             &Motion,
//             &Body,
//             &RayHits,
//         ),
//         With<Player>,
//     >,
//     player_collider_query: Query<Entity, With<PlayerColliderFlag>>,
//     mut ev_footstep: EventWriter<FootstepEvent>,
// ) {
//     if query.is_empty() || query.iter().len() > 1 {
//         warn!(
//             "Update Player Stance System found {} players, expected 1.",
//             query.iter().len()
//         );
//     }

//     for (
//         mut linear_vel,
//         mut external_force,
//         mut external_impulse,
//         mut gravity_scale,
//         mut stance,
//         motion,
//         body,
//         ray_hits,
//     ) in &mut query
//     {
//         // // We update stance_lockout.
//         // stance.lockout -= time.delta_secs();
//         // stance.lockout = f32::clamp(stance.lockout, 0.0, 1.0);

//         // // info!("ray_length: {}, ride_height: {}", ray_length, ride_height);

//         // let mut pad: Option<&Gamepad> = None;
//         // if let Ok((_entity, gamepad)) = gamepad_query.single() {
//         //     pad = Some(gamepad);
//         // }
//         // // Compute the next stance for the player.
//         // let next_stance: StanceType =
//         //     determine_next_stance(&keys, pad, &config, &mut stance, ray_length, ride_height);

//         // // handle footstep sound event when the state has changed and only then.
//         // if next_stance != stance.current {
//         //     match next_stance {
//         //         StanceType::Landing => {
//         //             // This is the sound effect that plays when the player has jumped or fallen and will land with both feet on the ground.
//         //             // this effect will play centered and will not pan in any direction.
//         //             ev_footstep.write(FootstepEvent {
//         //                 dir: FootstepDirection::None,
//         //                 volume: 1.0,
//         //             });
//         //         }
//         //         _ => (),
//         //     }
//         // }

//         // let next_gravity_scale: f32;

//         // match next_stance {
//         //     StanceType::Landing => {
//         //         // Set the gravity scale to zero.
//         //         next_gravity_scale = 0.0;
//         //     }
//         //     StanceType::Standing => {
//         //         // Set the gravity scale to zero.
//         //         next_gravity_scale = 0.0;
//         //         // Clear any persisting forces on the rigid body.
//         //         external_force.clear();
//         //         // lock the rotation
//         //     }
//         //     StanceType::Airborne => {
//         //         next_gravity_scale = 1.0;
//         //         // Clear any persisting forces on the rigid body.
//         //         external_force.clear();
//         //     }
//         //     StanceType::Jumping => {
//         //         // set the gravity scale to zero.
//         //         next_gravity_scale = 1.0;
//         //         // clear any persisting forces on the rigid body.
//         //         external_force.clear();
//         //         // check if the stance has changed.
//         //         if stance.current != StanceType::Jumping {
//         //             linear_vel.y = 0.0; // clear the jump velocity.
//         //             // apply_jump_force(
//         //             //     &config,
//         //             //     &mut external_impulse,
//         //             //     ray_length,
//         //             //     &mut stance,
//         //             //     &motion,
//         //             //     &body,
//         //             // );
//         //         }
//         //     }
//         // }

//         // // Update the gravity scale.
//         // gravity_scale.0 = next_gravity_scale;

//         // Update the current stance.
//         stance.current = next_stance.clone();
//     }
// }

fn determine_next_stance(
    keys: &Res<ButtonInput<KeyCode>>,
    gamepad: Option<&Gamepad>,
    config: &Res<PlayerControlConfig>,
    stance: &mut Stance,
    ray_length: f32,
    ride_height: f32,
) -> StanceType {
    let is_locked_out: bool = stance.lockout > 0.0;
    let previous_stance: StanceType = stance.current.clone();
    let mut next_stance: StanceType = stance.current.clone();

    let mut jump_pressed: bool = keys.pressed(KeyCode::Space);

    if let Some(g) = gamepad {
        if jump_pressed == false {
            jump_pressed = g.pressed(GamepadButton::North);
        }
    }

    // If your locked in you cannot change state.
    if !is_locked_out {
        if ray_length > ride_height + config.ray_length_offset {
            next_stance = StanceType::Airborne;
        } else if previous_stance == StanceType::Standing && stance.lockout <= 0.0 && jump_pressed {
            next_stance = StanceType::Jumping;
        } else if ray_length < ride_height {
            next_stance = StanceType::Standing;
        } else if previous_stance != StanceType::Standing
            && ray_length < ride_height + config.ray_length_offset
        {
            next_stance = StanceType::Landing;
        } else if ray_length > ride_height + config.ray_length_offset {
            next_stance = StanceType::Airborne;
        }
    }

    if next_stance != previous_stance {
        info!(
            "Stance Changed: {:#?} -> {:#?}",
            previous_stance, next_stance
        );
    }
    return next_stance;
}

#[derive(Component)]
pub struct StandingSpringForce {
    pub length: InterpolatedValue<f32>,
    pub max_extension: f32,
}

pub fn apply_standing_spring_force(
    config: Res<PlayerControlConfig>,
    mut query: Query<(
        Entity,
        &mut StandingSpringForce,
        &mut LinearVelocity,
        &mut ExternalForce,
        &mut GravityScale,
        &RayHits,
    )>,
    time: Res<Time>,
) {
    for (
        entity,
        mut standing_spring_force,
        mut linear_vel,
        mut external_force,
        mut gravity_scale,
        ray_hits,
    ) in &mut query
    {
        // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
        let mut ray_length: f32 = f32::INFINITY;

        // Find the first ray hit which is not its own collider.
        for hit in ray_hits.iter_sorted() {
            // ! BUG: We also need to ignore all child entities.
            if hit.entity != entity {
                ray_length = hit.distance;
                break;
            }
        }

        // Lerp current_ride_height to target_ride_height, this target_ride_height changes depending on the stance. Standing, Crouching, and Prone.
        standing_spring_force.length.current = exp_decay::<f32>(
            standing_spring_force.length.current,
            standing_spring_force.length.target,
            standing_spring_force.length.decay,
            time.delta_secs(),
        );

        let ride_height: f32 = standing_spring_force.length.current;
        let max_ray_length: f32 = standing_spring_force.length.current + standing_spring_force.max_extension;
        if ray_length <= max_ray_length {
            gravity_scale.0 = 0.0f32;

            external_force.clear();

            apply_spring_force(
                &config,
                &mut linear_vel,
                &mut external_force,
                ray_length,
                ride_height,
            );
        } else {
            gravity_scale.0 = 1.0f32;
        }
    }
}

pub fn lock_angular_velocity(mut query: Query<(&mut AngularVelocity, &Stance), With<Player>>) {
    for (mut angular_velocity, stance) in &mut query {
        match stance.current {
            StanceType::Standing | StanceType::Landing => {
                angular_velocity.0 = Vec3::ZERO;
            }
            _ => (),
        }
    }
}
