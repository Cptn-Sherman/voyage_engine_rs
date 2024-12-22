use bevy::{
    input::ButtonInput,
    prelude::{Commands, Entity, KeyCode, Query, Res, With},
    render::view::screenshot::{save_to_disk, Capturing, Screenshot},
    window::{SystemCursorIcon, Window},
    winit::cursor::CursorIcon,
};
use chrono::{DateTime, Local};

use crate::{
    config::{EngineSettings, KeyBindings},
    utils::{self, get_valid_extension},
};

pub mod config;
// Todo: at some point we should have the camera load its config from a toml file or generate the default, possibly using some perfromance calculator to determine what works best for the system in use.
pub mod camera;
pub mod flycam;

/** This system was taken from the screenshot example: https://bevyengine.org/examples/Window/screenshot/ */
pub fn take_screenshot(
    mut commands: Commands,
    settings: Res<EngineSettings>,
    key_bindings: Res<KeyBindings>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(key_bindings.screenshot_key) {
        // get the formated path as string.
        let date: DateTime<Local> = Local::now();
        let formated_date: chrono::format::DelayedFormat<chrono::format::StrftimeItems> =
            date.format("%Y-%m-%d_%H-%M-%S%.3f");
        let path: String = format!(
            "./voyage_screenshot-{}.{}",
            formated_date.to_string(),
            get_valid_extension(
                &settings.screenshot_format,
                utils::ExtensionType::Screenshot
            )
        );

        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn screenshot_saving(
    mut commands: Commands,
    screenshot_saving: Query<Entity, With<Capturing>>,
    windows: Query<Entity, With<Window>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    match screenshot_saving.iter().count() {
        0 => {
            commands.entity(window).remove::<CursorIcon>();
        }
        x if x > 0 => {
            commands
                .entity(window)
                .insert(CursorIcon::from(SystemCursorIcon::Progress));
        }
        _ => {}
    }
}
