use bevy::{asset::Handle, color::Color, text::{self, Font, TextSection, TextStyle}};

pub const DEFAULT_FONT_PATH: &str = "fonts/ashlanderPixel_fixed.ttf";
pub const DEFAULT_FONT_SIZE: f32 = 14.0;
pub const NO_PERCENTAGE: &str = "---.-%";

pub const ORANGE_TEXT_COLOR: Color = Color::hsv(0.34, 1.0, 0.5);
pub const YELLOW_GREEN_TEXT_COLOR: Color = Color::hsv(0.9, 0.69, 0.58);
pub const RED_TEXT_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub const GOLD_TEXT_COLOR: Color = Color::srgb(1.0 , 0.72, 0.0);
pub const BORDER_COLOR: Color = Color::srgb(0.8 , 0.8, 0.8);

pub fn get_text_section_array(strings: Vec<String>, style: TextStyle) -> Vec<TextSection> {
// create a list of gen_text_sections from a list of strings
    let text_sections = vec![
        // gen_text_section(Some("Hello".to_string()), None, None, font),
        // gen_text_section(Some("World".to_string()), None, None, font),
    ];
    text_sections
}

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