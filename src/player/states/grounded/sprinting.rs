use bevy::prelude::*;

use crate::player::{config::PlayerControlConfig, motion::Motion, stance::Stance, Player};


pub fn toggle_sprint(
    keys: Res<ButtonInput<KeyCode>>,
    player_config: Res<PlayerControlConfig>,
    mut query: Query<(&mut Motion, &Stance), With<Player>>,
) {
    for (mut motion, stance) in query.iter_mut() {
        // todo: replace with a crouch key binding.
        if !keys.pressed(KeyCode::ShiftLeft) {
            motion.sprinting = false;
            return;
        }

        motion.sprinting = true;

        if motion.sprinting == true {
            if stance.crouched == true {
                motion.current_movement_speed =
                player_config.movement_speed * 0.5 * player_config.sprint_speed_factor;
            } else {
                motion.current_movement_speed =
                    player_config.movement_speed * player_config.sprint_speed_factor;
            }
        } else {
            motion.current_movement_speed = player_config.movement_speed;
        }
    }
}
