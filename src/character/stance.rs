use super::{
    motion::{apply_jump_force, apply_spring_force, Motion},
    Player,
};
use crate::{ player::config::PlayerControlConfig, ternary, utils::exp_decay};
use avian3d::prelude::*;
use bevy::{
    asset::{AssetServer, Handle}, input::ButtonInput, log::{info, warn}, math::Vec3, prelude::{Commands, Component, Event, EventReader, EventWriter, KeyCode, Query, Res, ResMut, Resource, With}, time::Time, utils::info
};
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_turborand::{DelegatedRng, GlobalRng};
use crate::player::config::GetDownwardRayLengthMax;
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

impl StanceType {
    pub fn to_string(&self) -> &str {
        match self {
            StanceType::Airborne => "Airborne",
            StanceType::Standing => "Standing",
            StanceType::Landing => "Landing",
            StanceType::Jumping => "Jumping",
            StanceType::Crouching => "Crouching",
            StanceType::Crawling => "Crawling",
            StanceType::Prone => "Prone",
            StanceType::Sliding => "Sliding",
            StanceType::Vaulting => "Vaulting",
            StanceType::Hanging => "Hanging",
            StanceType::Climbing => "Climbing",
        }
    }
}

#[derive(Component)]
pub struct Stance {
    pub current: StanceType,
    pub crouched: bool,
    pub lockout: f32,
}

const PLAYBACK_RANGE: f64 = 0.4;

#[derive(Event, Clone)]
pub struct FootstepEvent {
    dir: FootstepDirection,

    volume: f64,
}

// this is the time in seconds between when the player takes a step. When running this is increased by the configured running speed multiplier.
// todo: When the ActionStep happens that is the point in time we apply a small impulse downward so the spring can have a lil' bump.
pub const ACTION_STEP_DELTA_DEFAULT: f32 = 0.60;

#[derive(Component)]
pub struct ActionStep {
    pub(crate) dir: FootstepDirection,
    pub(crate) bumped: bool,
    pub(crate) delta: f32,
}

pub(crate) fn tick_footstep(
    config: Res<PlayerControlConfig>,
    mut ev_footstep: EventWriter<FootstepEvent>,
    mut query: Query<(&mut ActionStep, &mut Motion, &Stance)>,
    time: Res<Time>,
) {
    for (mut action, mut motion, stance) in query.iter_mut() {
        // you must be on the ground for this sound to play.
        if stance.current != StanceType::Standing  {
            continue;
        }

        const LOCKIN_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.05;
        const BUMP_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.25;
        const BUMP_REMAINING_ACTION_STEP: f32 = ACTION_STEP_DELTA_DEFAULT * (1.0 - BUMP_ACTION_THRESHOLD_PERCENTAGE);
        const LOCKIN_ACTION_STEP_DELTA: f32 = ACTION_STEP_DELTA_DEFAULT * (1.0 - LOCKIN_ACTION_THRESHOLD_PERCENTAGE);
        // if you are not moving and need to take more than 85% of your remaining step we play no sound.
        if  motion.moving == false && action.delta >= LOCKIN_ACTION_STEP_DELTA {
            continue;
        }

        // scale the speed based on if you are sprinting or if you are not moving and are resting your foot.
        // when this value is higher you finish your step sooner.
        let mut scale: f32 = 1.0;
        let mut offset: f32 = ternary!(motion.sprinting, config.ride_height_step_offset, -config.ride_height_step_offset);
        if motion.sprinting  == true || motion.moving == false {
            scale = 1.45;
            offset *= 1.2; // this is kinda arbitrary.
        }

        // reduce the time by elaspsed times the scale.
        action.delta -= time.delta_seconds() * scale;
        let vol: f64 = ternary!(motion.moving, 0.5, 0.25);

        if action.delta <= BUMP_REMAINING_ACTION_STEP && action.bumped == false {
            motion.current_ride_height = config.ride_height + (offset * (2.0 * vol as f32));
            action.bumped = true;
        }

        // if the inter step delta has elapsed increase the delta, flip the dir, reset the bump, and queue the sound event.
        if action.delta <= 0.0 {
            action.delta += ACTION_STEP_DELTA_DEFAULT;
            action.dir = action.dir.flip();
            action.bumped = false; 
            ev_footstep.send(FootstepEvent {
                dir: action.dir.clone(),
                volume: vol,
            });
        }
    }
}
#[derive(Clone)]
pub enum FootstepDirection {
    None,
    Left,
    Right,
}

impl Default for FootstepDirection {
    fn default() -> Self {
        FootstepDirection::None
    }
}

impl FootstepDirection {
    fn value(&self) -> f64 {
        match self {
            FootstepDirection::None => 0.5,
            FootstepDirection::Left => 0.3,
            FootstepDirection::Right => 0.7,
        }
    }

    fn flip(&self) -> Self {
        match self {
            FootstepDirection::None => FootstepDirection::None,
            FootstepDirection::Left => FootstepDirection::Right,
            FootstepDirection::Right => FootstepDirection::Left,
        }
    }
}

#[derive(Resource)]
pub struct MyAudioHandle(Handle<AudioSource>);

pub fn load_footstep_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handle = asset_server.load("audio\\footstep-fx.mp3");
    commands.insert_resource(MyAudioHandle(handle.clone()));
}

// ! This should ideally not take in and load a new sound ever time and should be loaded once. ALSO, remove the inability to iterate over all the events this should be solved with an update.
// ! ALSO GENERALIZE THIS TO ANY SOUND.
// ! You should only need to send panning, volume and a sound effect tag to get the right one and it looks up from asset map or some shit.
pub fn play_footstep_sfx(
    mut ev_footstep: EventReader<FootstepEvent>,
    mut global_rng: ResMut<GlobalRng>,
    audio: Res<Audio>,
    my_audio_handle: Res<MyAudioHandle>,
) {
    let mut should_play: bool = false;
    let mut panning: f64 = 0.5;
    let mut volume: f64 = 0.5;

    for ev in ev_footstep.read() {
        should_play = true;
        panning = ev.dir.value();
        volume = ev.volume;
    }

    if should_play {
        let random_playback_rate: f64 = global_rng.f64() * PLAYBACK_RANGE + 0.8;
        audio
            .into_inner()
            .play(my_audio_handle.0.clone())
            .with_panning(panning)
            .with_playback_rate(random_playback_rate)
            .with_volume(volume);
    }
}


// todo: I want to try making it faster to move "forward" and slower to move left, right or backwards. Maybe we construct a const movement speed scaler for each direction.
pub fn update_player_stance(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<PlayerControlConfig>,
    mut query: Query<
        (
            &mut LinearVelocity,
            &mut ExternalForce,
            &mut ExternalImpulse,
            &mut GravityScale,
            &mut Stance,
            &mut Motion,
            &RayCaster,
            &RayHits,
        ),
        With<Player>,
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
        mut stance,
        mut motion,
        caster,
        ray_hits,
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
        let next_stance: StanceType = determine_next_stance(&keys, &config, &mut stance, ray_length);

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
                apply_spring_force(&config, &mut linear_vel, &mut external_force, ray_length, motion.current_ride_height);
            }
            StanceType::Standing => {
                // Set the gravity scale to zero.
                next_gravity_scale = 0.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();

                
                
                apply_spring_force(&config, &mut linear_vel, &mut external_force, ray_length, motion.current_ride_height);
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
                    apply_jump_force(&config, &mut stance, &mut external_impulse, &mut linear_vel, ray_length);
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

        // Lerp current_ride_height back to normal ride_height. Right now this assumes "normal" is standing.
        motion.current_ride_height = exp_decay(motion.current_ride_height, motion.target_ride_height, 6.0, time.delta_seconds());
        info!("Current Ride Height: {}", motion.current_ride_height);
        // Update the gravity scale.
        gravity_scale.0 = next_gravity_scale;

        // Update the current stance.
        stance.current = next_stance.clone();
    }
}

fn determine_next_stance(
    keys: &Res<ButtonInput<KeyCode>>,
    config: &Res<PlayerControlConfig>,
    stance: &mut Stance,
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
