#[macro_export]
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
// macro_rules! double_for_loop {
//     ($var1:ident in $iter1:expr, $var2:ident in $iter2:expr, $callback:expr) => {
//         for $var1 in $iter1 {
//             for $var2 in $iter2 {
//                 $callback
//             }
//         }
//     };
// }
use bevy::{
    asset::{Assets, Handle},
    input::ButtonInput,
    log::{info, warn},
    math::{EulerRot, Quat, Vec2, Vec3},
    prelude::{KeyCode, Mesh, Query, Res, ResMut, With},
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        renderer::RenderAdapter,
    },
    window::{CursorGrabMode, PrimaryWindow, Window},
};
use std::fmt::Write;

use crate::Bindings;

/// Formats a value as a string with optional decimal digits and support for negative space formatting.
///
/// # Arguments
///
/// * `value` - The value to format.
/// * `decimal_digits` - The number of decimal digits to include. Pass `Some(digits)` for a specific number of digits, or `None` for no decimal digits or `Some(0)`.
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
/// let neg_formatted_neg_buffered = format_value_f32(-42.0, None, true);
/// assert_eq!(neg_formatted_neg_buffered, "-42");
/// let pos_formatted_neg_buffered = format_value_f32(42.0, None, true);
/// assert_eq!(pos_formatted_neg_buffered, " 42");
/// let neg_formatted_unbuffered = format_value_f32(-42.0, None, false);
/// assert_eq!(neg_formatted_neg_buffered, "-42");
/// let pos_formatted_unbuffered = format_value_f32(42.0, None, false);
/// assert_eq!(pos_formatted_neg_buffered, "42");
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

    let width = num_digits + decimal_digits.unwrap_or(0);

    if format_negative_space && value >= 0.0 {
        write!(
            &mut buffer,
            " {:>width$.decimal_width$}", // <--- this line has a single extra space, its hard to see :)
            value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    } else {
        write!(
            &mut buffer,
            "{:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    }
    .expect("Failed to write to buffer while formatting value as string!");

    buffer
}

pub fn format_value_vec3(
    vec: Vec3,
    decimal_digits: Option<usize>,
    format_negative_space: bool,
) -> String {
    return format!(
        "[{}, {}, {}]",
        format_value_f32(vec.x, decimal_digits, format_negative_space),
        format_value_f32(vec.y, decimal_digits, format_negative_space),
        format_value_f32(vec.z, decimal_digits, format_negative_space)
    );
}

pub fn format_value_quat(
    quat: Quat,
    decimal_digits: Option<usize>,
    format_negative_space: bool,
    output_euler: Option<EulerRot>
) -> String {
    match output_euler {
        None => {
            return format!(
                "[{}, {}, {}, {}]",
                format_value_f32(quat.x, decimal_digits, format_negative_space),
                format_value_f32(quat.y, decimal_digits, format_negative_space),
                format_value_f32(quat.z, decimal_digits, format_negative_space),
                format_value_f32(quat.w, decimal_digits, format_negative_space)
            );
        },
        _  => {
            let (yaw, pitch, roll) = quat.to_euler(output_euler.unwrap());
            return format!(
                "[{}, {}, {}]",
                format_value_f32(yaw, decimal_digits, format_negative_space),
                format_value_f32(pitch, decimal_digits, format_negative_space),
                format_value_f32(roll, decimal_digits, format_negative_space),
            );
        }
    }
}

pub fn format_percentage_f32(value: Option<f32>) -> Option<String> {
    match value {
        Some(v) if v >= 0.0 && v <= 100.0 => Some(format!("{: >5.1}%", v)),
        _ => None,
    }
}

pub fn format_percentage_f64(value: Option<f64>) -> Option<String> {
    match value {
        Some(v) if v >= 0.0 && v <= 100.0 => Some(format!("{: >5.1}%", v)),
        _ => None,
    }
}



pub fn generate_plane_mesh(
    meshes: &mut ResMut<Assets<Mesh>>,
    width: f32,
    length: f32,
    uv_scale: f32,
) -> Handle<Mesh> {
    let half_width = width / 2.0;
    let half_length = length / 2.0;

    let vertices = vec![
        // Top face
        (
            [-half_width, 0.0, half_length],
            [0.0, 1.0, 0.0],
            [0.0, uv_scale * length],
        ), // Top-left
        (
            [half_width, 0.0, half_length],
            [0.0, 1.0, 0.0],
            [uv_scale * width, uv_scale * length],
        ), // Top-right
        (
            [half_width, 0.0, -half_length],
            [0.0, 1.0, 0.0],
            [uv_scale * width, 0.0],
        ), // Bottom-right
        (
            [-half_width, 0.0, -half_length],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
        ), // Bottom-left
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, // top face
    ];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    for (position, normal, uv) in vertices {
        positions.push(position);
        normals.push(normal);
        uvs.push(uv);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(positions),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::from(uvs));
    mesh.insert_indices(Indices::U32(indices));

    meshes.add(
        mesh.with_generated_tangents()
            .expect("Failed to generate tangents for the mesh"),
    )
}

// --- Validating file extensions ---
const VALID_SCREENSHOT_EXTENSIONS: [&str; 3] = ["png", "jpeg", "bmp"];
const VALID_VIDEO_EXTENSIONS: [&str; 3] = ["mp4", "avi", "mkv"];

pub enum ExtensionType {
    Screenshot,
    Video,
}

pub fn get_valid_extension<'a>(extension: &'a str, ext_type: ExtensionType) -> &'a str {
    let valid_extensions = match ext_type {
        ExtensionType::Screenshot => &VALID_SCREENSHOT_EXTENSIONS,
        ExtensionType::Video => &VALID_VIDEO_EXTENSIONS,
    };

    let default_extension = match ext_type {
        ExtensionType::Screenshot => "png",
        ExtensionType::Video => "mp4",
    };

    if valid_extensions.contains(&extension.to_lowercase().as_str()) {
        extension
    } else {
        default_extension
    }
}

// Pulled this from Freya Holmer's Lerp smoothing is broken talk. https://www.youtube.com/watch?v=LSNQuFEDOyQ
pub fn exp_decay(a: f32, b: f32, decay: f32, delta_time: f32) -> f32 {
    return b + (a - b) * (-decay * delta_time).exp();
}

pub fn exp_vec3_decay(a: Vec3, b: Vec3, decay: f32, delta_time: f32) -> Vec3 {
    return b + (a - b) * (-decay * delta_time).exp();
}

pub fn increase_render_adapter_wgpu_limits(render_adapter: Res<RenderAdapter>) {
    render_adapter
        .limits()
        .max_sampled_textures_per_shader_stage = 32;
    info!(
        "max_sampled_textures_per_shader_stage is {} ",
        render_adapter
            .limits()
            .max_sampled_textures_per_shader_stage
    );
}

// Start up system used to capture the mouse. 
// ! There is currently a bug in the x11 implementation which causes this to fail on linux and sets the window to monitor 0.
pub fn capture_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        // Check if the cursor is already grabbed
        if window.cursor_options.grab_mode != CursorGrabMode::Locked {
            toggle_cursor_grab_mode(&mut window);
        }
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

pub fn detect_toggle_cursor(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<Bindings>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = primary_window.single_mut() {
        if keys.just_pressed(key_bindings.action_toggle_cursor_focus) {
            toggle_cursor_grab_mode(&mut window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}



fn toggle_cursor_grab_mode(window: &mut Window) {
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            set_cursor_grab_mode(window, CursorGrabMode::Confined, true);
        }
        _ => {
            set_cursor_grab_mode(window, CursorGrabMode::None, true);
        }
    }
}

fn set_cursor_grab_mode(window: &mut Window, grab_mode: CursorGrabMode, center_cursor: bool) {
    window.cursor_options.grab_mode = grab_mode;
    window.cursor_options.visible = ternary!(grab_mode == CursorGrabMode::None, true, false);

    if center_cursor {
        // set the cursor to the center of the screen.
        let window_width = (window.width() / 2.0) + window.ime_position.x;
        let window_height = (window.height() / 2.0) + window.ime_position.y;
        window.set_cursor_position(Some(Vec2::new(window_width / 2.0, window_height / 2.0)));
    }
}
