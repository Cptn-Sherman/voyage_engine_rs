use avian3d::collision::Collider;
use bevy::prelude::*;

use crate::{config::KeyBindings, player::{body::Body, config::PlayerControlConfig, motion::Motion, stance::Stance, Player}};

pub fn toggle_crouching(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    player_config: ResMut<PlayerControlConfig>,
    mut query: Query<(&mut Body, &mut Stance, &mut Motion, &mut Collider), With<Player>>,
) {
    for (mut body, mut stance, mut motion, mut collider) in query.iter_mut() {

        if !keys.just_pressed(key_bindings.toggle_crouched) {
            return;
        }

        // Toggle crouching flag
        stance.crouched = !stance.crouched;
        info!("Toggled crouching to: {}", stance.crouched);

        // Update the collider scale
        if stance.crouched == true {
            let crouched_height = player_config.capsule_height * player_config.crouched_height_factor;
            
            collider.set_scale(Vec3::from([1.0, crouched_height, 1.0]), 4);
            body.current_body_height = crouched_height;
            
            motion.target_ride_height = player_config.ride_height * player_config.crouched_height_factor;
            motion.current_movement_speed = player_config.movement_speed / 2.0;
        } else {
            collider.set_scale(Vec3::from([1.0, player_config.capsule_height, 1.0]), 4);
            body.current_body_height = player_config.capsule_height;
            
            motion.target_ride_height = player_config.ride_height;
            motion.current_movement_speed = player_config.movement_speed;
        }

        info!("Updated collider scale to: {:?}", collider.scale());
    }
}
