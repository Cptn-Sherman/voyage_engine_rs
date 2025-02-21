use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::{config::KeyBindings, player::{body::Body, config::PlayerControlConfig, stance::Stance, Player, PlayerColliderFlag}};

pub fn toggle_crouching(
    mut player_query: Query<(&mut Body, &mut Stance), With<Player>>,
    mut player_collider_query:  Query<&mut Collider, With<PlayerColliderFlag>>, // , (With<PlayerCollider>, With<PlayerColliderFlag>, Without<Player>)
    player_config: ResMut<PlayerControlConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
) {
    for (mut body, mut stance) in player_query.iter_mut() {
        if !keys.just_pressed(key_bindings.toggle_crouched) {
            return;
        }

        let  mut  collider = player_collider_query.single_mut();
        
        // Toggle crouching flag
        stance.crouched = !stance.crouched;
        
        if stance.crouched == true {
            // Update the collider scale
            let crouched_height: f32 = player_config.capsule_height * player_config.crouched_height_factor;
            collider.set_scale(Vec3::from([1.0, crouched_height, 1.0]), 10);
            stance.target_ride_height = player_config.ride_height * player_config.crouched_height_factor;
            body.current_body_height = crouched_height;
        } else {
            // Reset the collider scale to One
            collider.set_scale(Vec3::from([1.0, 1.0, 1.0]), 10);
            stance.target_ride_height = player_config.ride_height;
            body.current_body_height = player_config.capsule_height;
        }
        
        info!("Updated: Crouched -> {}, Collider scaled to: {:?}", stance.crouched, collider.scale());
    }
}