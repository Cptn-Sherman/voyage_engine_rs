


// basic setup for bevy plugin
use bevy::prelude::*;

struct Crosshair;

pub fn create_crosshair(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    location: Vec2,
) {
    // load the crosshair texture
    let texture_handle = asset_server.load("textures/crosshair.png");

    // create the crosshair sprite
    
}