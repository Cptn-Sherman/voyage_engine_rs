use bevy::{asset::Handle, color::Color, text::{Font, TextSection, TextStyle}};

pub const DEFAULT_FONT_PATH: &str = "fonts/Monocraft.ttf";
pub const DEFAULT_FONT_SIZE: f32 = 18.0;
pub const NO_PERCENTAGE: &str = "---.-%";

pub const ORANGE_TEXT_COLOR: Color = Color::hsv(0.34, 1.0, 0.5);
pub const YELLOW_GREEN_TEXT_COLOR: Color = Color::hsv(0.9, 0.69, 0.58);
pub const RED_TEXT_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const GOLD_TEXT_COLOR: Color = Color::srgb(1.0 , 0.72, 0.0);
pub const BORDER_COLOR: Color = Color::srgb(0.8 , 0.8, 0.8);


pub fn gen_text_section(
    value: Option<String>,
    size: Option<f32>,
    color: Option<Color>,
    font: Handle<Font>,
) -> TextSection {
    TextSection::new(
        value.unwrap_or("".to_string()),
        TextStyle {
            font,
            font_size: size.unwrap_or(DEFAULT_FONT_SIZE),
            color: color.unwrap_or(Color::WHITE),
        },
    )
}