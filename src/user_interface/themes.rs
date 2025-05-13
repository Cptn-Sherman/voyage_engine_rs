use bevy::{asset::Handle, color::Color, prelude::{Bundle, Text}, text::{ Font, FontSmoothing, LineHeight, TextColor, TextFont}};

pub const DEFAULT_FONT_PATH: &str = "fonts/AshlanderPixel_fixed.ttf";
pub const DEFAULT_DEBUG_FONT_PATH: &str = "fonts/mononoki-Bold.ttf";
pub const DEFAULT_FONT_SIZE: f32 = 14.0;
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
) -> impl Bundle { 
    (
        Text::new(value.unwrap_or_default()),
        TextFont {
            font,
            font_size: size.unwrap_or(DEFAULT_FONT_SIZE),
            font_smoothing: FontSmoothing::AntiAliased,
            line_height: LineHeight::default(),
        },
        TextColor(color.unwrap_or(Color::WHITE)),
    )
}

// pub fn get_text_style() -> TextStyle {
//     TextStyle {
//         font: Handle::Weak(Font::default()),
//         font_size: DEFAULT_FONT_SIZE,
//         color: Color::WHITE,
//     }
// }

enum TextType {
    Default,
}