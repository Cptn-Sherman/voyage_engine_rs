use bevy::prelude::*;

use crate::{config::KeyBindings, player::{config::PlayerControlConfig, motion::Motion, stance::{Stance, StanceType}, Player}};


pub fn toggle_sprint(
    mut query: Query<(&mut Motion, &Stance), With<Player>>,
    player_config: Res<PlayerControlConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
) {
    for (mut motion, stance) in query.iter_mut() {
        
        if stance.current == StanceType::Airborne {
            //info!("Skipping while airborne, Motion Speed Locked at: {}", motion.current_movement_speed);
            return;
        }
        
        motion.sprinting = keys.pressed(key_bindings.toggle_sprint);
        
        if motion.sprinting == true {
            if stance.crouched == true {
                motion.target_movement_speed =
                player_config.movement_speed * 0.5 * player_config.sprint_speed_factor;
            } else {
                motion.target_movement_speed =
                    player_config.movement_speed * player_config.sprint_speed_factor;
            }
        } else {
            motion.target_movement_speed = player_config.movement_speed;
        }
        //info!("Motion Speed: {}", motion.current_movement_speed);
    }
}
