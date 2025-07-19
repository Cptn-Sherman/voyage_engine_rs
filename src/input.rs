use bevy::{ecs::{entity::Entity, query::With, resource::Resource, system::{Query, Res, ResMut}}, input::{gamepad::{Gamepad, GamepadAxis}, keyboard::KeyCode, mouse::AccumulatedMouseMotion, ButtonInput}, log::info, math::Vec3, window::{PrimaryWindow, Window}};

use crate::{config::Bindings, player::config::PlayerControlConfig, utils::format_value_vec3};

// todo: make this adjustable in the config.
const ANALOGE_STICK_DEADZONE: f32 = 0.1;

#[derive(Resource)]
pub struct Input {
    pub movement: Vec3,
    pub direction: Vec3,
}

pub fn update_input_resource(
    mut input: ResMut<Input>,
    accumulated_mouse_motion: ResMut<AccumulatedMouseMotion>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    gamepads: Query<(Entity, &Gamepad)>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<PlayerControlConfig>,
    key_bindings: Res<Bindings>,
) {
    // this is the raw input vector
    input.movement = Vec3::ZERO.clone();
    input.direction = Vec3::ZERO.clone();

    if keys.pressed(key_bindings.move_forward) {
        input.movement.z = 1.0;
    }
    if keys.pressed(key_bindings.move_backward) {
        input.movement.z = -1.0;
    }
    if keys.pressed(key_bindings.move_left) {
        input.movement.x = -1.0;
    }
    if keys.pressed(key_bindings.move_right) {
        input.movement.x = 1.0;
    }

    input.direction.x = config.mouse_look_sensitivity * accumulated_mouse_motion.delta.x;
    input.direction.y = config.mouse_look_sensitivity * accumulated_mouse_motion.delta.y;

    if let Ok((_entity, gamepad)) = gamepads.single() {
        let left_stick_x: f32 = gamepad.get(GamepadAxis::LeftStickX).unwrap_or_default();
        let left_stick_y: f32 = gamepad.get(GamepadAxis::LeftStickY).unwrap_or_default();
        let right_stick_x: f32 = gamepad.get(GamepadAxis::RightStickX).unwrap_or_default();
        let right_stick_y: f32 = gamepad.get(GamepadAxis::RightStickY).unwrap_or_default();

        if left_stick_x.abs() > ANALOGE_STICK_DEADZONE {
            input.movement.x = left_stick_x;
        }

        if left_stick_y.abs() > ANALOGE_STICK_DEADZONE {
            input.movement.y = left_stick_y;
        }

        if let Ok(window) = primary_window.single() { 
            let window_scale: f32 = window.height().min(window.width());

            if right_stick_x.abs() > ANALOGE_STICK_DEADZONE {
                input.direction.x = config.gamepad_look_sensitivity * right_stick_x * window_scale
            }
    
            if right_stick_y.abs() > ANALOGE_STICK_DEADZONE {
                input.direction.y = config.gamepad_look_sensitivity * right_stick_y * window_scale
            }
        }
    }

    info!("Movement: {}, Direction: {}", format_value_vec3(input.movement, Some(2) , true), format_value_vec3(input.direction, Some(2) , true));
}