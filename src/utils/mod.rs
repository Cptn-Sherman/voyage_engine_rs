use bevy::{
    asset::{Assets, Handle, RenderAssetUsages},
    ecs::{
        entity::Entity,
        system::{Commands, Single},
    },
    input::ButtonInput,
    log::{info, warn},
    math::{f32, Vec2},
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::{KeyCode, Mesh, Query, Res, ResMut, With},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, Window},
};

use std::ops::{Add, Mul, Sub};

use crate::Bindings;

pub mod format_value;

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

// * --- Cursor Grab ---
// Start up system used to capture the mouse.
// ! There is currently a bug in the x11 implementation which causes this to fail on linux and sets the window to monitor 0.
// ! The initial cursor grab will succeed but the center will fail.
pub fn initial_grab_cursor(cursor_options: Single<&mut CursorOptions>) {
    set_cursor_grab_mode(cursor_options, CursorGrabMode::Locked);
}

pub fn initial_cursor_center(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        center_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_cursor_center`!");
    }
}

pub fn detect_toggle_cursor_system(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    cursor_options: Single<&mut CursorOptions>,
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<Bindings>,
) {
    if let Ok(mut window) = primary_window.single_mut() {
        if keys.just_pressed(key_bindings.action_toggle_cursor_focus) {
            toggle_cursor_grab_mode(&mut window, cursor_options);
        }
    } else {
        warn!("Primary window not found for `detect_toggle_cursor`!");
    }
}

fn set_cursor_grab_mode(mut cursor_options: Single<&mut CursorOptions>, grab_mode: CursorGrabMode) {
    cursor_options.grab_mode = grab_mode;
    cursor_options.visible = ternary!(grab_mode == CursorGrabMode::None, true, false);
    info!(
        "Setting window grab mode: {}",
        grab_mode_stringified(&grab_mode)
    );
}

// Sets the cursor to the center of the window.
pub fn center_cursor(window: &mut Window) {
    let center: Vec2 = Vec2 {
        x: window.width() / 2.,
        y: window.height() / 2.,
    };
    window.set_cursor_position(Some(center));
}

fn toggle_cursor_grab_mode(window: &mut Window, cursor_options: Single<&mut CursorOptions>) {
    match cursor_options.grab_mode {
        CursorGrabMode::None => {
            set_cursor_grab_mode(cursor_options, CursorGrabMode::Locked);
            center_cursor(window);
        }
        _ => {
            set_cursor_grab_mode(cursor_options, CursorGrabMode::None);
        }
    }
}

fn grab_mode_stringified(grab_mode: &CursorGrabMode) -> String {
    match grab_mode {
        CursorGrabMode::Confined => "Confined".to_string(),
        CursorGrabMode::Locked => "Locked".to_string(),
        CursorGrabMode::None => "None".to_string(),
    }
}

// Close the focused window whenever the escape key (Esc) is pressed
// This is useful for examples or prototyping.
pub fn close_on_key(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<Bindings>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(key_bindings.action_close_application) {
            commands.entity(window).despawn();
        }
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
