use crate::player::config::GetDownwardRayLengthMax;
use crate::{player::config::PlayerControlConfig, ternary, utils::exp_decay};
use avian3d::prelude::*;
use bevy::{
    asset::{AssetServer, Handle},
    input::ButtonInput,
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

use super::body::Body;
use super::motion::{apply_jump_force, apply_spring_force, Motion};
use super::Player;

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

// This is the time in seconds between each footstep. When sprinting this value is multiplied.
pub const ACTION_STEP_DELTA_DEFAULT: f32 = 0.50;
const LOCKIN_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.05;
const BUMP_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.25;
const BUMP_REMAINING_ACTION_STEP: f32 =
    ACTION_STEP_DELTA_DEFAULT * (1.0 - BUMP_ACTION_THRESHOLD_PERCENTAGE);
const LOCKIN_ACTION_STEP_DELTA: f32 =
    ACTION_STEP_DELTA_DEFAULT * (1.0 - LOCKIN_ACTION_THRESHOLD_PERCENTAGE);

#[derive(Component)]
pub struct ActionStep {
    pub(crate) dir: FootstepDirection,
    pub(crate) bumped: bool,
    pub(crate) delta: f32,
}

pub(crate) fn tick_footstep(
    mut ev_footstep: EventWriter<FootstepEvent>,
    mut query: Query<(&mut ActionStep, &mut Motion, &Stance)>,
    config: Res<PlayerControlConfig>,
    time: Res<Time>,
) {
    for (mut action, mut motion, stance) in query.iter_mut() {

        if motion.moving == true {
            info!("you are moving");
        }
        // you must be on the ground for this sound to play.
        if stance.current != StanceType::Standing {
            continue;
        }
        // if you are not moving and need to take more than 85% of your remaining step we play no sound.
        if motion.moving == false && action.delta >= LOCKIN_ACTION_STEP_DELTA
        {
            continue;
        }

        // scale the speed based on if you are sprinting or if you are not moving and are resting your foot.
        // when this value is higher you finish your step sooner.
        let mut step_speed_scale: f32 = 1.0;
        let mut ride_height_offset: f32 = ternary!(
            motion.sprinting,
            config.ride_height_step_offset,
            -config.ride_height_step_offset
        );

        if motion.sprinting == true || motion.moving == false {
            step_speed_scale = 1.45;
            ride_height_offset *= 1.2; // this is kinda arbitrary. but this little bit of kick is applied when you start sprinting from a stand still.
        }

        // reduce the time by elaspsed times the scale.
        action.delta -= time.delta_secs() * step_speed_scale;
        let vol: f64 = ternary!(motion.moving, 0.5, 0.25);
        let current_ride_height_offset_scaler: f32 = ternary!(motion.moving, 1.0, 0.5);

        // bump the riding height when the delta is less than the bump threshold.
        if config.enable_view_bobbing
            && action.delta <= BUMP_REMAINING_ACTION_STEP
            && action.bumped == false
        {
            motion.current_ride_height =
                config.ride_height + (ride_height_offset * current_ride_height_offset_scaler);
            action.bumped = true;
        }

        // if the inter step delta has elapsed increase the delta, flip the dir, reset the bump, and queue the sound event.
        if action.delta <= 0.0 {
            // send the play sound event.
            ev_footstep.send(FootstepEvent {
                dir: action.dir.clone(),
                volume: vol,
            });
            // reset the delta.
            action.delta += ACTION_STEP_DELTA_DEFAULT;
            // reset the bumped flag.
            action.bumped = false;
            // flip the direction of the footstep panning.
            action.dir = action.dir.flip();
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


// todo: update this to use constants so you can customize the offset from each ear. 
// Maybe obsolete if a 3D sound implementation is used instead. Would be nice for ui. 
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
pub struct FootstepAudioHandle(Handle<AudioSource>);

pub fn load_footstep_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("audio\\footstep-fx.mp3");
    commands.insert_resource(FootstepAudioHandle(handle.clone()));
}

// todo: move this somewhere more appropriate.
// ! This should ideally not take in and load a new sound ever time and should be loaded once. ALSO, remove the inability to iterate over all the events this should be solved with an update.
// ! ALSO GENERALIZE THIS TO ANY SOUND.
// ! You should only need to send panning, volume and a sound effect tag to get the right one and it looks up from asset map or some shit...
pub fn play_footstep_sfx(
    mut ev_footstep: EventReader<FootstepEvent>,
    mut global_rng: ResMut<GlobalRng>,
    audio: Res<Audio>,
    my_audio_handle: Res<FootstepAudioHandle>,
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
            &mut Body,
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
        body,
        ray_hits,
    ) in &mut query
    {
        // We update stance_lockout.
        stance.lockout -= time.delta_secs();
        stance.lockout = f32::clamp(stance.lockout, 0.0, 1.0);

        // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
        let mut ray_length: f32 = f32::INFINITY;
        if let Some(hit) = ray_hits.iter_sorted().next() {
            ray_length = hit.distance;
        }

        // Compute the next stance for the player.
        let next_stance: StanceType =
            determine_next_stance(&keys, &config, &mut stance, ray_length);

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
                motion.current_ride_height = config.ride_height * 0.85;
                apply_spring_force(
                    &config,
                    &mut linear_vel,
                    &mut external_force,
                    ray_length,
                    motion.current_ride_height,
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
                    motion.current_ride_height,
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
                        &motion,
                        &body,
                    );
                }
            }
        }

        // Lerp current_ride_height to target_ride_height, this target_ride_height changes depending on the stance. Standing, Crouching, and Prone.
        motion.current_ride_height = exp_decay(
            motion.current_ride_height,
            motion.target_ride_height,
            6.0,
            time.delta_secs(),
        );

        // Update the gravity scale.
        gravity_scale.0 = next_gravity_scale;

        // Update the current stance.
        stance.current = next_stance.clone();
    }
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
            "Stance Changed: {:#?} -> {:#?}",
            previous_stance, next_stance
        );
    }
    return next_stance;
}
