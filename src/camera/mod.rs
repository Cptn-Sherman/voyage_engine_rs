use bevy::{input::ButtonInput, log::{error, info}, prelude::{Entity, KeyCode, Query, Res, ResMut, With}, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
use chrono::{DateTime, Local};

use crate::{config::{EngineSettings, KeyBindings}, utils::{self, get_valid_extension}};

pub mod config;

// Todo: at some point we should have the camera load its config from a toml file or generate the default, possibly using some perfromance calculator to determine what works best for the system in use.
pub mod camera;
pub mod flycam;


/** This system was taken from the screenshot example: https://bevyengine.org/examples/Window/screenshot/ */
pub fn take_screenshot(
    settings: Res<EngineSettings>,
    key_bindings: Res<KeyBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    if keys.just_pressed(key_bindings.screenshot_key) {
        // get the formated path as string.
        let date: DateTime<Local> = Local::now();
        let formated_date: chrono::format::DelayedFormat<chrono::format::StrftimeItems> =
            date.format("%Y-%m-%d_%H-%M-%S%.3f");
        let path: String = format!(
            "./voyage_screenshot-{}.{}",
            formated_date.to_string(),
            get_valid_extension(&settings.format, utils::ExtensionType::Screenshot)
        );

        // attempt to save the screenshot to disk and bubble up.
        match screenshot_manager.save_screenshot_to_disk(main_window.single(), path) {
            Ok(_) => info!("Screenshot saved successfully."),
            Err(e) => {
                error!("Failed to save screenshot: {}", e);
            }
        }
    }
}