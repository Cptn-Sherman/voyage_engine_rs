use avian3d::collision::Collider;
use bevy::prelude::*;

use crate::{character::{motion::Motion, stance::Stance, Player}, player::config::PlayerControlConfig};

// ! bug: right now the player can crouch and will collide with the terrain. Which causes them to fallover and get stuck on the ground.  

pub fn toggle_crouching(
    keys: Res<ButtonInput<KeyCode>>,
    player_config: Res<PlayerControlConfig>,
    mut query: Query<(&mut Stance, &mut Motion, &mut Collider), With<Player>>,
) {
    for (mut stance, mut motion, mut collider) in query.iter_mut() {
        // Check if the control key is pressed
        // todo: replace with a crouch key binding.
        if !keys.just_pressed(KeyCode::ControlLeft) {
            return;
        }

        // Toggle crouching flag
        stance.crouched = !stance.crouched;
        info!("Toggled crouching to: {}", stance.crouched);

        // Update the collider scale
        if stance.crouched == true {
            collider.set_scale(Vec3::from([1.0, 0.4, 1.0]), 4);
            motion.target_ride_height = player_config.ride_height * 0.75;
            motion.current_movement_speed = player_config.movement_speed / 2.0;
        } else {
            collider.set_scale(Vec3::from([1.0, 1.0, 1.0]), 4);
            motion.target_ride_height = player_config.ride_height;
            motion.current_movement_speed = player_config.movement_speed;
        }
    }
}
