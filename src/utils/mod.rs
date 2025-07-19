use bevy::{
    asset::{Assets, Handle},
    input::ButtonInput,
    log::warn,
    math::{f32, EulerRot, Quat, Vec2, Vec3},
    prelude::{KeyCode, Mesh, Query, Res, ResMut, With},
    render::{
        mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
        render_asset::RenderAssetUsages,
    },
    window::{CursorGrabMode, PrimaryWindow, Window},
};

use std::{
    fmt::{Display, Write},
    ops::{Add, Mul, Sub},
};

use crate::{user_interface::themes::NO_PERCENTAGE, Bindings};

#[macro_export]
macro_rules! ternary {
    ($condition:expr, $true_expr:expr, $false_expr:expr) => {
        if $condition {
            $true_expr
        } else {
            $false_expr
        }
    };
}

pub struct InterpolatedValue<T>
where
    T: Copy + Sub<Output = T> + Mul<f32, Output = T> + Add<Output = T>,
{
    pub current: T,
    pub target: T,
    pub decay: f32,
}

impl<T> InterpolatedValue<T>
where
    T: Copy + Sub<Output = T> + Mul<f32, Output = T> + Add<Output = T>,
{
    pub fn new(initial: T, decay: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            decay,
        }
    }
}

// Pulled this from Freya Holmer's Lerp smoothing is broken talk. https://www.youtube.com/watch?v=LSNQuFEDOyQ
pub fn exp_decay<T>(a: T, b: T, decay: f32, delta_time: f32) -> T
where
    T: Copy + Sub<Output = T> + Mul<f32, Output = T> + Add<Output = T>,
{
    b + (a - b) * (-decay * delta_time).exp()
}

// todo: Add bool for restrict range to keep between 0.0-100.0.
// todo: Add config for number of decimal places.
// todo: Add config for number of numerial places, the number of digits to the right side of the dot.
// todo: Replace "None" with something like nan% or better.
pub fn format_percentage<T>(value: T) -> String
where
    T: Display + PartialOrd + Into<f64>,
{
    let v: f64 = value.into();
    if v >= 0.0 && v <= 100.0 {
        format!("{: >5.1}%", v)
    } else {
        NO_PERCENTAGE.to_owned()
    }
}

pub fn format_value_f32(
    value: f32,
    decimal_digits: Option<usize>,
    format_negative_space: bool,
) -> String {
    let mut buffer: String = String::new();

    let rounded_value: i32 = value as i32;

    let num_digits: usize = if rounded_value == 0 {
        1 // Account for the single digit zero
    } else {
        rounded_value.to_string().len() // Calculate the number of digits
    };

    let width: usize = num_digits + decimal_digits.unwrap_or(0);

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
    output_euler: Option<EulerRot>,
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
        }
        _ => {
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

// * --- Cursor Grab ---
// Start up system used to capture the mouse.
// ! There is currently a bug in the x11 implementation which causes this to fail on linux and sets the window to monitor 0.
pub fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
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
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<Bindings>,
) {
    if let Ok(mut window) = primary_window.single_mut() {
        if keys.just_pressed(key_bindings.action_toggle_cursor_focus) {
            toggle_cursor_grab_mode(&mut window);
        }
    } else {
        warn!("Primary window not found for `detect_toggle_cursor`!");
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

// * --- Generate Meshes ---
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

// * --- Valid File Extensions ---
const VALID_EXTENSIONS_VIDEO: [&str; 3] = ["mp4", "avi", "mkv"];
const VALID_EXTENSIONS_SCREENSHOT: [&str; 3] = ["png", "jpeg", "bmp"];

pub enum ExtensionType {
    Screenshot,
    _Video,
}

pub fn get_valid_extension<'a>(extension: &'a str, ext_type: ExtensionType) -> &'a str {
    let valid_extensions = match ext_type {
        ExtensionType::Screenshot => &VALID_EXTENSIONS_SCREENSHOT,
        ExtensionType::_Video => &VALID_EXTENSIONS_VIDEO,
    };

    let default_extension = match ext_type {
        ExtensionType::Screenshot => "png",
        ExtensionType::_Video => "mp4",
    };

    if valid_extensions.contains(&extension.to_lowercase().as_str()) {
        extension
    } else {
        default_extension
    }
}

// pub fn increase_render_adapter_wgpu_limits(render_adapter: Res<RenderAdapter>) {
//     render_adapter
//         .limits()
//         .max_sampled_textures_per_shader_stage = 32;
//     info!(
//         "max_sampled_textures_per_shader_stage is {} ",
//         render_adapter
//             .limits()
//             .max_sampled_textures_per_shader_stage
//     );
// }
