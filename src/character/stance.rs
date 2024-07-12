use super::{
    motion::{apply_jump_force, apply_spring_force},
    Config, PlayerControl,
};
use crate::character::GetDownwardRayLengthMax;
use avian3d::prelude::*;
use bevy::{
    asset::AssetServer,
    input::ButtonInput,
    log::{info, warn},
    math::Vec3,
    prelude::{Component, Event, EventReader, EventWriter, KeyCode, Query, Res, ResMut, With},
    time::Time,
};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_turborand::{DelegatedRng, GlobalRng};

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


const PLAYBACK_RANGE: f64 = 0.4;

#[derive(Event)]
pub struct FootstepEvent;

pub fn play_footstep_sfx(mut ev_footstep: EventReader<FootstepEvent>, asset_server: Res<AssetServer>, audio: Res<Audio>, mut global_rng: ResMut<GlobalRng>) {
    let mut should_play = false;
    for _ev in ev_footstep.read() {
        info!("FOOTSTEP!");
        should_play = true;
    }
    if should_play {
        let random_playback_rate: f64 = global_rng.into_inner().f64() * PLAYBACK_RANGE + 0.8;
        audio
        .into_inner()
        .play(asset_server.load("audio\\footstep-fx.mp3"))
        .with_panning(0.3)
        .with_playback_rate(random_playback_rate)
        .with_volume(0.5);
    }
}

pub fn update_player_stance(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
    mut query: Query<
        (
            &mut LinearVelocity,
            &mut ExternalForce,
            &mut ExternalImpulse,
            &mut GravityScale,
            &mut RayCaster,
            &RayHits,
            &mut Stance,
        ),
        With<PlayerControl>,
    >,
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
        mut caster,
        ray_hits,
        mut stance,
    ) in &mut query
    {
        // We update stance_lockout.
        stance.lockout -= time.delta_seconds();
        stance.lockout = f32::clamp(stance.lockout, 0.0, 1.0);

        // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
        let mut ray_length: f32 = f32::INFINITY;
        if let Some(hit) = ray_hits.iter_sorted().next() {
            ray_length = Vec3::length(caster.direction * hit.time_of_impact);
        }

        // Compute the next stance for the player.
        let next_stance: StanceType = determine_next_stance(&keys, &config, &stance, ray_length);

        // handle footstep sound event when the state has changed and only then.
        if next_stance != stance.current {
            match next_stance {
                StanceType::Landing => {
                    // Play first the step sound effect.
                    // we are garenteed that this is the first tick were this new stance is equal to this value.
                    ev_footstep.send(FootstepEvent);
                }
                StanceType::Standing => {
                    if stance.current == StanceType::Landing {
                        // we play the second standing soundeffect here.
                        //ev_footstep.send(FootstepEvent);
                    }
                }
                _ => {
                    // do nothing...
                }
            }
        }
        let next_gravity_scale: f32;

        match next_stance {
            StanceType::Landing => {
                // Set the gravity scale to zero.
                next_gravity_scale = 0.0;
                apply_spring_force(&config, &mut linear_vel, &mut external_force, ray_length);
            }
            StanceType::Standing => {
                // Set the gravity scale to zero.
                next_gravity_scale = 0.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();

                apply_spring_force(&config, &mut linear_vel, &mut external_force, ray_length);
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
        gravity_scale.0 = next_gravity_scale;

        // Update the current stance.
        stance.current = next_stance.clone();
    }
}

fn determine_next_stance(
    keys: &Res<ButtonInput<KeyCode>>,
    config: &Res<Config>,
    stance: &Stance,
    ray_length: f32,
) -> StanceType {
    let is_locked_out: bool = stance.lockout > 0.0;
    let previous_stance: StanceType = stance.current.clone();
    let mut next_stance: StanceType = stance.current.clone();
    // If your locked in you cannot change state.
    if !is_locked_out {
        if ray_length > config.get_downard_ray_length_max() {
            next_stance = StanceType::Airborne;
        } else if previous_stance == StanceType::Standing
            && stance.lockout <= 0.0
            && keys.pressed(KeyCode::Space)
        {
            next_stance = StanceType::Jumping;
        } else if ray_length < config.ride_height {
            next_stance = StanceType::Standing;
        } else if previous_stance != StanceType::Standing
            && ray_length < config.get_downard_ray_length_max()
        {
            next_stance = StanceType::Landing;
        } else if ray_length > config.get_downard_ray_length_max() {
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
