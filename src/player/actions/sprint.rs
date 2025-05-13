use bevy::prelude::*;

use crate::{
    config::Bindings,
    player::{
        motion::Motion,
        stance::{Stance, StanceType},
        Player,
    },
};

pub fn toggle_sprinting(
    mut player_query: Query<(&mut Motion, &Stance), With<Player>>,
    gamepad_query: Query<(Entity, &Gamepad)>,
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<Bindings>,
) {
    for (mut motion, stance) in player_query.iter_mut() {
        if stance.current == StanceType::Airborne {
            return;
        }

        if let Ok((_entity, gamepad)) = gamepad_query.single() {
            motion.sprinting = gamepad.pressed(bindings.action_sprint.button);
        } else {
            motion.sprinting = keys.pressed(bindings.action_sprint.key);
        }
    }
}
