use bevy::prelude::*;

use crate::{config::KeyBindings, player::{motion::Motion, stance::{Stance, StanceType}, Player}};


pub fn toggle_sprinting(
    mut player_query: Query<(&mut Motion, &Stance), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
) {
    for (mut motion, stance) in player_query.iter_mut() {
        if stance.current == StanceType::Airborne {
            return;
        }
        
        motion.sprinting = keys.pressed(key_bindings.toggle_sprint);
    }
}
