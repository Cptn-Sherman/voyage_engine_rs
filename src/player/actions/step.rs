use bevy::{
    asset::{AssetServer, Handle}, core_pipeline::core_3d::Camera3d, ecs::{
        component::Component, event::{Event, EventReader, EventWriter}, query::{With, Without}, system::{Commands, Query, Res, ResMut}
    }, math::{EulerRot, Vec3}, time::Time, transform::components::Transform
};
use bevy::prelude::Resource;
use bevy_kira_audio::{Audio, AudioControl, AudioSource};
use bevy_turborand::{DelegatedRng, GlobalRng};

use crate::{
    player::{
        config::PlayerControlConfig,
        motion::{Motion, LEAN_LOCKOUT_TIME, ROTATION_AMOUNT},
        stance::{Stance, StanceType}, Player,
    },
    ternary,
};

const PLAYBACK_RANGE: f64 = 0.4;

#[derive(Event, Clone)]
pub struct FootstepEvent {
    pub(crate) dir: FootstepDirection,

    pub(crate) volume: f64,
}

// this is the time in seconds between when the player takes a step. When running this is increased by the configured running speed multiplier.
// todo: When the ActionStep happens that is the point in time we apply a small impulse downward so the spring can have a lil' bump.

// This is the time in seconds between each footstep. When sprinting this value is multiplied.
pub const ACTION_STEP_DELTA_DEFAULT: f32 = 0.64;
const LOCKIN_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.1;
const BUMP_ACTION_THRESHOLD_PERCENTAGE: f32 = 0.70;
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

#[derive(Clone, PartialEq)]
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

const FOOTSTEP_CENTER: f64 = 0.5;
const FOOTSTEP_OFFSET: f64 = 0.05;

impl FootstepDirection {
    fn value(&self) -> f64 {
        match self {
            FootstepDirection::None => FOOTSTEP_CENTER,
            FootstepDirection::Left => FOOTSTEP_CENTER - FOOTSTEP_OFFSET,
            FootstepDirection::Right => FOOTSTEP_CENTER + FOOTSTEP_OFFSET,
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
    let handle = asset_server.load("audio/Concrete20.wav");
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

pub fn tick_footstep(
    mut ev_footstep: EventWriter<FootstepEvent>,
    mut query: Query<(&mut ActionStep, &mut Stance, &mut Motion), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Player>)>,
    player_config: Res<PlayerControlConfig>,
    config: Res<PlayerControlConfig>,
    time: Res<Time>,
) {
    for (mut action, mut stance, mut motion) in query.iter_mut() {
       
        // you must be on the ground for this sound to play.
        if stance.current != StanceType::Standing && stance.current != StanceType::Landing {
            continue;
        }
        // if you are not moving and need to take more than 85% of your remaining step we play no sound.
        if motion.moving == false && action.delta >= LOCKIN_ACTION_STEP_DELTA {
            continue;
        }

        // scale the speed based on if you are sprinting or if you are not moving and are resting your foot.
        // when this value is higher you finish your step sooner.

        let step_speed_scale: f32 = motion.movement_speed.current / player_config.default_movement_speed;

        // info!("Step Speed Scale: {}", step_speed_scale);

        let mut ride_height_offset: f32 = ternary!(
            motion.sprinting,
            config.ride_height_step_offset,
            -config.ride_height_step_offset
        );

        if motion.sprinting == true || motion.moving == false {
            ride_height_offset *= 1.4; // this is kinda arbitrary. but this little bit of kick is applied when you start sprinting from a stand still.
        }

        // reduce the time by elaspsed times the scale.
        action.delta -= time.delta_secs() * step_speed_scale;
        let mut vol: f64 = ternary!(motion.moving, 0.75, 0.50);
        if motion.sprinting {
            vol += 0.15;
        }
        let current_ride_height_offset_scaler: f32 = ternary!(motion.moving, 1.0, 0.5);

        // bump the riding height when the delta is less than the bump threshold.
        if config.enable_view_bobbing
            && action.delta <= BUMP_REMAINING_ACTION_STEP
            && action.bumped == false
        {
            stance.ride_height.current =
                config.ride_height + (ride_height_offset * current_ride_height_offset_scaler);
            action.bumped = true;
            let camera_transform = camera_query.single_mut().unwrap();
            let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::default());
            //let pitch = input_vector.y * rotation_amount.to_radians();
            let dir: f32 = ternary!(action.dir == FootstepDirection::Left, 1.0, -1.0);
            let sprinting_scale: f32 = ternary!(motion.sprinting, 0.2, 0.15);
            let roll: f32 = dir * ROTATION_AMOUNT.to_radians() * sprinting_scale;
            motion.target_lean = Vec3::from_array([yaw, pitch, roll]);
            motion.lock_lean = LEAN_LOCKOUT_TIME;
        }

        // if the inter step delta has elapsed increase the delta, flip the dir, reset the bump, and queue the sound event.
        if action.delta <= 0.0 {
            // send the play sound event.
            ev_footstep.write(FootstepEvent {
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
