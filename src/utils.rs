pub const CHUNK_SIZE_F32: f32 = 16.0;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE_F32 as i32;
pub const CHUNK_SIZE_F32_MIDPOINT: f32 = CHUNK_SIZE_F32 / 2.0;
pub const CHUNK_SIZE_I32_MIDPOINT: i32 = CHUNK_SIZE_F32_MIDPOINT as i32;

/// Macro for emulating a ternary operator in Rust.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// let x = 42;
/// let y = ternary!(x > 0, "positive", "non-positive");
/// println!("y: {}", y); // Output: y: positive
/// ```
///
/// # Syntax
///
/// The `ternary` macro takes three arguments:
///
/// * `$condition` - An expression that evaluates to a boolean condition.
/// * `$true_expr` - An expression to be evaluated if the condition is true.
/// * `$false_expr` - An expression to be evaluated if the condition is false.
///
/// # Notes
///
/// The `ternary` macro expands into an `if` statement that evaluates the condition and returns the corresponding expression.
///
/// It is important to note that macros should be used judiciously, considering the readability and maintainability of the code.
macro_rules! ternary {
    ($condition:expr, $true_expr:expr, $false_expr:expr) => {
        if $condition {
            $true_expr
        } else {
            $false_expr
        }
    };
}


/// Macro for creating a double for loop with a function callback.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// let numbers = vec![1, 2, 3];
/// let characters = vec!['A', 'B', 'C'];
///
/// double_for_loop!(num in &numbers, ch in &characters, {
///     println!("Number: {}, Character: {}", num, ch);
/// });
/// ```
///
/// # Syntax
///
/// The `double_for_loop` macro takes three arguments:
///
/// * `$var1` - An identifier to represent the current element in the first iterator.
/// * `in $iter1` - The first iterator to iterate over.
/// * `$var2` - An identifier to represent the current element in the second iterator.
/// * `in $iter2` - The second iterator to iterate over.
/// * `$callback` - A block of code to execute for each combination of elements from the two iterators.
///
/// # Notes
///
/// The `double_for_loop` macro expands into a nested for loop that iterates over the given iterators and invokes the provided callback for each combination of values.
///
/// It is important to note that macros should be used judiciously, considering the readability and maintainability of the code.
macro_rules! double_for_loop {
    ($var1:ident in $iter1:expr, $var2:ident in $iter2:expr, $callback:expr) => {
        for $var1 in $iter1 {
            for $var2 in $iter2 {
                $callback
            }
        }
    };
}


use std::fmt::Write;
use std::cmp::PartialOrd;
use bevy::{prelude::{Image, info}, render::{render_resource::{Extent3d, TextureDimension, TextureFormat, SamplerDescriptor}, texture::ImageSampler}};
use num_traits::{Zero, Float};

/// Formats a value as a string with optional decimal digits and support for negative space formatting.
///
/// # Arguments
///
/// * `value` - The value to format.
/// * `decimal_digits` - The number of decimal digits to include. Pass `Some(digits)` for a specific number of digits, or `None` for no decimal digits.
/// * `format_negative_space` - Determines whether negative values should be padded with a leading space.
///
/// # Returns
///
/// A formatted string representation of the value.
///
/// # Examples
///
/// Formatting a positive value with 2 decimal places:
///
/// ```rust
/// let formatted = format_value(3.14, Some(2), false);
/// assert_eq!(formatted, " 3.14");
/// ```
///
/// Formatting a negative value without any decimal places:
///
/// ```rust
/// let formatted = format_value(-42, None, true);
/// assert_eq!(formatted, "-42");
/// ```

pub fn format_value_f32(
    value: f32,
    decimal_digits: Option<usize>,
    format_negative_space: bool,
) -> String {
    let mut buffer = String::new();

    let rounded_value = value as i32;

    let num_digits = if rounded_value == 0 {
        1 // Account for the single digit zero
    } else {
        rounded_value.to_string().len() // Calculate the number of digits
    };

    let width = if rounded_value >= 0 || !format_negative_space {
        num_digits + decimal_digits.unwrap_or(0) // Add one extra space for positive values and decimal digits
    } else {
        num_digits + 1 + decimal_digits.unwrap_or(0) // Add two extra spaces for negative values (including the negative sign) and decimal digits
    };

    if format_negative_space && rounded_value >= 0 {
        write!(
            &mut buffer,
            " {:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    } else {
        write!(
            &mut buffer,
            "{:>width$.decimal_width$}",
            rounded_value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    }
    .expect("Failed to write to buffer");

    buffer
}



/// Converts a coordinate to a chunk coordinate.
///
/// Chunks are square regions in a 2D grid. This function takes a coordinate
/// and returns the corresponding chunk coordinate. The chunk coordinate
/// represents the index of the chunk that contains the given coordinate.
///
/// # Arguments
///
/// * `coord` - The coordinate value to convert.
///
/// # Returns
///
/// The chunk coordinate that corresponds to the given coordinate.
///
/// # Examples
///
/// ```
/// let coord = -15;
/// let chunk_coord = convert_to_chunk_coordinate(coord);
/// assert_eq!(chunk_coord, -1);
/// ```
pub fn convert_to_chunk_coordinate(coord: i32) -> i32 {
    if coord < 0 {
        (coord + 1) / (CHUNK_SIZE_F32 as i32) - 1
    } else {
        coord / CHUNK_SIZE_F32 as i32
    }
}


/// Creates a colorful test pattern.
///
/// This function generates a debug texture with a colorful test pattern. It creates an image with a specified size
/// and fills it with a palette of predefined colors. Each row of the image is filled with a rotated version of the
/// palette, creating a visually appealing pattern.
///
/// # Returns
///
/// An `Image` object representing the generated debug texture.
pub fn uv_debug_texture() -> Image {
    info!("Generating Debug Texture");

    // Define the size of the texture
    const TEXTURE_SIZE: usize = 8;

    // Define the palette of colors
    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    // Create the texture data array to store pixel information
    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];

    // Generate the test pattern by filling the texture with rotated palette colors
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    // Create an `Image` object with the generated texture data
    let mut img = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );

    // Set the sampler descriptor for the image
    img.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor::default());

    // Return the generated image
    img
}