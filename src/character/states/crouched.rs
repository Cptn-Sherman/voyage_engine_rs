use avian3d::{
    collision::Collider,
    parry::shape::{self, SharedShape},
};
use bevy::prelude::*;

use crate::character::{motion::Motion, stance::Stance, Config, Player};

pub fn toggle_crouching(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
    mut query: Query<(&mut Stance, &mut Motion, &mut Collider), With<Player>>,
) {
    for (mut stance, mut motion, mut collider) in query.iter_mut() {
        // Check if the control key is pressed
        if !keys.just_pressed(KeyCode::ControlLeft) {
            return;
        }

        // Toggle crouching flag
        stance.crouched = !stance.crouched;
        info!("Toggled crouching to: {}", stance.crouched);

        // Update the collider scale
        if stance.crouched == true {
            collider.set_scale(Vec3::from([1.0, 0.5, 1.0]), 4);
            motion.target_ride_height = config.ride_height / 2.0;
        } else {
            collider.set_scale(Vec3::from([1.0, 1.0, 1.0]), 4);
            motion.target_ride_height = config.ride_height;
        }
    }
}