use crate::{player::config::PlayerControlConfig, ternary, utils::exp_decay};
use avian3d::prelude::*;
use bevy::{
    asset::{AssetServer, Handle},
    ecs::entity::Entity,
    input::{
        gamepad::{Gamepad, GamepadButton},
        ButtonInput,
    },
    log::{info, warn},
    math::{Quat, Vec3},
    prelude::{
        Commands, Component, Event, EventReader, EventWriter, KeyCode, Query, Res, ResMut,
        Resource, With,
    },
    time::Time,
};
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_turborand::{DelegatedRng, GlobalRng};

use super::{actions::step::{ActionStep, FootstepDirection, FootstepEvent}, motion::{apply_jump_force, apply_spring_force, Motion}};
use super::Player;
use super::{body::Body, PlayerColliderBundle, PlayerColliderFlag};

#[derive(Debug, PartialEq, Clone)]
// each of these stance types needs to have a movement speed calculation, a
pub enum StanceType {
    Airborne,
    Standing,
    Landing,
    Jumping,
}

impl StanceType {
    pub fn to_string(&self) -> &str {
        match self {
            StanceType::Airborne => "Airborne",
            StanceType::Standing => "Standing",
            StanceType::Landing => "Landing",
            StanceType::Jumping => "Jumping",
        }
    }
}

#[derive(Component)]
pub struct Stance {
    pub(crate) current_ride_height: f32,
    pub(crate) target_ride_height: f32,
    pub current: StanceType,
    pub crouched: bool,
    pub lockout: f32,
}

// todo: I want to try making it faster to move "forward" and slower to move left, right or backwards. Maybe we construct a const movement speed scaler for each direction.
pub fn update_player_stance(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<PlayerControlConfig>,
    gamepad_query: Query<(Entity, &Gamepad)>,
    mut query: Query<
        (
            &mut LinearVelocity,
            &mut ExternalForce,
            &mut ExternalImpulse,
            &mut GravityScale,
            &mut Stance,
            &mut Body,
            &RayHits,
        ),
        With<Player>,
    >,
    player_collider_query: Query<Entity, With<PlayerColliderFlag>>,
    mut ev_footstep: EventWriter<FootstepEvent>,
) {
    if query.is_empty() || query.iter().len() > 1 {
        warn!(
            "Update Player Stance System found {} players, expected 1.",
            query.iter().len()
        );
    }

    for (
        mut linear_vel,
        mut external_force,
        mut external_impulse,
        mut gravity_scale,
        mut stance,
        body,
        ray_hits,
    ) in &mut query
    {
        // We update stance_lockout.
        stance.lockout -= time.delta_secs();
        stance.lockout = f32::clamp(stance.lockout, 0.0, 1.0);

        // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
        let mut ride_height: f32 = stance.current_ride_height;
        let mut ray_length: f32 = f32::INFINITY;

        // Find the first ray hit which is not the player collider.
        for hit in ray_hits.iter_sorted() {
            if hit.entity != player_collider_query.single() {
                ray_length = hit.distance;
                break;
            }
        }

        // info!("ray_length: {}, ride_height: {}", ray_length, ride_height);
        
        let mut pad: Option<&Gamepad> = None;
        if let Ok((_entity, gamepad)) = gamepad_query.get_single() {
            pad = Some(gamepad);
        }
        // Compute the next stance for the player.
        let next_stance: StanceType = determine_next_stance(
            &keys,
            pad,
            &config,
            &mut stance,
            ray_length,
            ride_height,
        );

        // handle footstep sound event when the state has changed and only then.
        if next_stance != stance.current {
            match next_stance {
                StanceType::Landing => {
                    // This is the sound effect that plays when the player has jumped or fallen and will land with both feet on the ground.
                    // this effect will play centered and will not pan in any direction.
                    ev_footstep.send(FootstepEvent {
                        dir: FootstepDirection::None,
                        volume: 1.0,
                    });
                }
                _ => (),
            }
        }

        let next_gravity_scale: f32;

        match next_stance {
            StanceType::Landing => {
                // Set the gravity scale to zero.
                next_gravity_scale = 0.0;
                ride_height *= 0.85;
                apply_spring_force(
                    &config,
                    &mut linear_vel,
                    &mut external_force,
                    ray_length,
                    ride_height,
                );
            }
            StanceType::Standing => {
                // Set the gravity scale to zero.
                next_gravity_scale = 0.0;
                // Clear any persisting forces on the rigid body.
                external_force.clear();
                // lock the rotation

                apply_spring_force(
                    &config,
                    &mut linear_vel,
                    &mut external_force,
                    ray_length,
                    ride_height,
                );
            }
            StanceType::Airborne => {
                next_gravity_scale = 1.0;
                // Clear any persisting forces on the rigid body.
                external_force.clear();
            }
            StanceType::Jumping => {
                // set the gravity scale to zero.
                next_gravity_scale = 1.0;
                // clear any persisting forces on the rigid body.
                external_force.clear();
                // check if the stance has changed.
                if stance.current != StanceType::Jumping {
                    linear_vel.y = 0.0; // clear the jump velocity.
                    apply_jump_force(
                        &config,
                        &mut stance,
                        &mut external_impulse,
                        &mut linear_vel,
                        ray_length,
                        &body,
                    );
                }
            }
        }

        // Lerp current_ride_height to target_ride_height, this target_ride_height changes depending on the stance. Standing, Crouching, and Prone.
        stance.current_ride_height = exp_decay(
            stance.current_ride_height,
            stance.target_ride_height,
            6.0,
            time.delta_secs(),
        );

        // Update the gravity scale.
        gravity_scale.0 = next_gravity_scale;

        // Update the current stance.
        stance.current = next_stance.clone();
    }
}

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

    let mut jump_pressed = keys.pressed(KeyCode::Space);

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


pub fn lock_rotation(
    mut query: Query<(&mut AngularVelocity, &mut Rotation, &mut Stance), With<Player>>,
) {
    for (mut angular_velocity, mut rotation, stance) in &mut query {
        match stance.current {
            StanceType::Standing | StanceType::Landing => {
                rotation.0 = Quat::IDENTITY;
                angular_velocity.0 = Vec3::ZERO;
            }
            _ => (),
        }
    }
}

