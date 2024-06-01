use bevy::{
    app::{App, Plugin, Startup, Update}, asset::Assets, ecs::{
        bundle::Bundle, component::Component, entity::Entity, event::{Events, ManualEventReader}, query::{Added, With, Without}, schedule::States, system::{Commands, Query, Res, ResMut, Resource}
    }, input::{keyboard::KeyCode, mouse::MouseMotion, Input}, log::{info, warn}, math::{EulerRot, Quat, Vec3}, pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial}, prelude::default, render::{
        camera::Camera,
        color::Color,
        mesh::{shape, Mesh},
    }, time::Time, transform::components::Transform, window::{CursorGrabMode, PrimaryWindow, Window}
};


use bevy_xpbd_3d::{
    components::{
        Collider, ExternalForce, ExternalImpulse, GravityScale, LinearVelocity, Mass, RigidBody,
    },
    plugins::spatial_query::{RayCaster, RayHits},
};

use crate::{toggle_grab_cursor, KeyBindings};

const CAPSULE_HEIGHT: f32 = 1.0;
const RIDE_HEIGHT: f32 = 1.5;
const RAY_LENGTH_OFFSET: f32 = 0.5;
const DOWNWARD_RAY_LENGTH_MAX: f32 = RAY_LENGTH_OFFSET + RIDE_HEIGHT;
const RIDE_SPRING_STRENGTH: f32 = 800.0;
const RIDE_SPRING_DAMPER: f32 = 75.0;
const DEFAULT_STANCE_LOCKOUT: f32 = 0.25;
const JUMP_STRENGTH: f32 = 130.0;

const MOVEMENT_SPEED: f32 = 50.0;
const MOVEMENT_DECAY: f32 = 0.95;

const LOOK_SENSITIVITY: f32 = 0.00012;

pub struct FirstPersonPlayerControllerPlugin;

#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum CameraState {
    #[default]
    FirstPersonCamera,
    FreeCamera,
    PanOrbit,
}

impl Plugin for FirstPersonPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing player controller plugin");
        app.init_resource::<InputState>()
            .init_resource::<KeyBindings>();
        app.add_systems(
            Startup,
            (
                spawn_player_system,
                initial_grab_on_player_spawn,
            ),
        );
        app.add_systems(
            Update,
            (
                update_player_data_system,
                camera_look_system,
                attached_camera_system,
            ),
        );
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub rigid_body: RigidBody,
    pub mass: Mass,
    pub gravity_scale: GravityScale,
    pub collider: Collider,
    pub mat_mesh_bundle: MaterialMeshBundle<StandardMaterial>,
    pub data: PlayerData,
}

#[derive(Component)]
pub struct PlayerData {
    gravity_scale: f32,
    current_stance: PlayerStance,
    stance_lockout: f32,
    movement_vec: Vec3,
}

#[derive(Debug, PartialEq, Clone)]
enum PlayerStance {
    Standing,
    Landing,
    Jumping,
    Airborne,
}

fn spawn_player_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Spawning player contorller...");
    commands.spawn((
        PlayerBundle {
            rigid_body: RigidBody::Dynamic,
            mass: Mass(20.0),
            gravity_scale: GravityScale(1.0),
            collider: Collider::capsule(CAPSULE_HEIGHT, 0.5),
            mat_mesh_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(0.0, 8.0, 0.0),
                ..default()
            },
            data: PlayerData {
                gravity_scale: 1.0,
                current_stance: PlayerStance::Airborne,
                stance_lockout: 0.0,
                movement_vec: Vec3::new(0.0, 0.0, 0.0),
            },
        },
        RayCaster::new(Vec3::ZERO, Vec3::NEG_Y),
    ));
}

fn update_player_data_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut camera_query: Query<(&Transform, With<Camera>)>,
    mut query: Query<(
        &RayCaster,
        &RayHits,
        &mut LinearVelocity,
        &mut GravityScale,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut PlayerData,
    )>,
) {
    for (
        ray,
        hits,
        mut vel,
        mut gravity,
        mut external_force,
        mut external_impulse,
        mut data,
    ) in &mut query
    {
        for (camera_transform, _) in &mut camera_query {
            // We update stance_lockout.
            data.stance_lockout -= time.delta_seconds();
            data.stance_lockout = f32::clamp(data.stance_lockout, 0.0, 1.0);

            // Compute the ray_length to a hit, if we don't hit anything we assume the ground is infinitly far away.
            let mut ray_length: f32 = f32::INFINITY;
            if let Some(hit) = hits.iter_sorted().next() {
                ray_length = Vec3::length(ray.direction * hit.time_of_impact);
            }

            // Compute the next stance for the player.
            let next_stance: PlayerStance = determine_stance(&keys, &data, ray_length);

            match next_stance {
                PlayerStance::Landing => {
                    // Set the gravity scale to zero.
                    data.gravity_scale = 0.0;
                    apply_spring_force(&mut external_force, ray_length, vel.y);
                }
                PlayerStance::Standing => {
                    // Set the gravity scale to zero.
                    data.gravity_scale = 0.0;

                    // Clear any persisting forces on the rigid body.
                    external_force.clear();

                    apply_spring_force(&mut external_force, ray_length, vel.y);
                }
                PlayerStance::Airborne => {
                    // Set the gravity scale to zero.
                    data.gravity_scale = 1.0;

                    // Clear any persisting forces on the rigid body.
                    external_force.clear();
                }
                PlayerStance::Jumping => {
                    // set the gravity scale to zero.
                    data.gravity_scale = 1.0;

                    // clear any persisting forces on the rigid body.
                    external_force.clear();

                    // check if the stance has changed.
                    if data.current_stance != PlayerStance::Jumping {
                        vel.y = 0.0; // clear the jump velocity.
                        apply_jump_force(&mut data, &mut external_impulse, ray_length);
                    }
                }
            }

            // --- Movement ---

            // Perform the movement checks.
            let mut movement_vector: Vec3 = Vec3::ZERO.clone();
            let speed_vector: Vec3 = Vec3::from([MOVEMENT_SPEED, MOVEMENT_SPEED, MOVEMENT_SPEED]);

            if keys.pressed(key_bindings.move_forward) {
                movement_vector += camera_transform.forward();
            }
            if keys.pressed(key_bindings.move_backward) {
                movement_vector += camera_transform.back();
            }
            if keys.pressed(key_bindings.move_left) {
                movement_vector += camera_transform.left();
            }
            if keys.pressed(key_bindings.move_right) {
                movement_vector += camera_transform.right();
            }

            // apply the total movement vector.
            data.movement_vec +=
                movement_vector.normalize_or_zero() * speed_vector * time.delta_seconds();
            // Appy decay to Linear Velocity on the X and Z directions and apply to the velocity.
            data.movement_vec.x *= MOVEMENT_DECAY;
            data.movement_vec.z *= MOVEMENT_DECAY;
            vel.x = data.movement_vec.x;
            vel.z = data.movement_vec.z;

            // --- State Update ---

            // Update the gravity scale.
            gravity.0 = data.gravity_scale;

            // Update the current stance.
            data.current_stance = next_stance.clone();
        }
    }
}

fn attached_camera_system(
    player_query: Query<&mut Transform, With<PlayerData>>,
    mut camera_query: Query<(&mut Transform, With<Camera>, Without<PlayerData>)>,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        info!("The camera attach system did not recieve 1 player and 1 camera.");
    }

    for (mut camera_transform, _, _) in &mut camera_query {
        for player_transform in &player_query {
            camera_transform.translation = player_transform.translation.clone();
            //camera_transform.rotation = player_transform.rotation.into();
        }
    }
}

// This function and many of its helpers are ripped from, bevy_fly_cam.
fn camera_look_system(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(window) = primary_window.get_single() {
        let delta_state = state.as_mut();
        for mut transform in camera_query.iter_mut() {
            for ev in delta_state.reader_motion.read(&motion) {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let window_scale = window.height().min(window.width());
                        pitch -= (LOOK_SENSITIVITY * ev.delta.y * window_scale).to_radians();
                        yaw -= (LOOK_SENSITIVITY * ev.delta.x * window_scale).to_radians();
                        pitch = pitch.clamp(-1.54, 1.54);
                    }
                }
                // prevent the camera from looping over itself in pitch only.
                pitch = pitch.clamp(-1.54, 1.54);
                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    }
}

// Grab cursor when an entity with FlyCam is added
fn initial_grab_on_player_spawn(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    query_added: Query<Entity, Added<PlayerData>>,
) {
    if query_added.is_empty() {
        return;
    }

    if let Ok(window) = &mut primary_window.get_single_mut() {
        toggle_grab_cursor(window);
        info!("Cursor was grabbed!");
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn determine_stance(
    keys: &Res<Input<KeyCode>>,
    data: &PlayerData,
    ray_length: f32,
) -> PlayerStance {
    let is_locked_out: bool = data.stance_lockout > 0.0;
    let previous_stance: PlayerStance = data.current_stance.clone();
    let mut next_stance: PlayerStance = data.current_stance.clone();
    // If your locked in you cannot change state.
    if !is_locked_out {
        if ray_length > DOWNWARD_RAY_LENGTH_MAX {
            next_stance = PlayerStance::Airborne;
        } else if previous_stance == PlayerStance::Standing
            && data.stance_lockout <= 0.0
            && keys.pressed(KeyCode::Space)
        {
            next_stance = PlayerStance::Jumping;
        } else if ray_length < RIDE_HEIGHT {
            next_stance = PlayerStance::Standing;
        } else if previous_stance != PlayerStance::Standing && ray_length < DOWNWARD_RAY_LENGTH_MAX
        {
            next_stance = PlayerStance::Landing;
        } else if ray_length > DOWNWARD_RAY_LENGTH_MAX {
            next_stance = PlayerStance::Airborne;
        }
    }

    if next_stance != previous_stance {
        info!(
            "Detected Stance Change:{:#?} -> {:#?}",
            previous_stance, next_stance
        );
    }

    return next_stance;
}

fn apply_spring_force(force: &mut ExternalForce, ray_length: f32, velocity_y: f32) {
    // Find the diference between how close the capsule is to the surface beneath it.
    // Compute this value by subtracting the ray length from the set ride height
    // to find the diference in position.
    let spring_offset = f32::abs(ray_length) - RIDE_HEIGHT;
    let spring_force = (spring_offset * RIDE_SPRING_STRENGTH) - (-velocity_y * RIDE_SPRING_DAMPER);

    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
    force.clear();
    force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
}

fn apply_jump_force(data: &mut PlayerData, impulse: &mut ExternalImpulse, ray_length: f32) {
    // Apply the stance cooldown now that we are jumping
    data.stance_lockout = DEFAULT_STANCE_LOCKOUT;

    let half_jump_strength: f32 = JUMP_STRENGTH / 2.0;
    let jump_factor: f32 = compute_clamped_jump_force_factor(ray_length);

    // make this value changable.
    let dynamic_jump_strength: f32 = half_jump_strength + (half_jump_strength * jump_factor);

    // todo: right now we are applying this jump force directly up, this needs to consider the original movement velocities.
    // maybe instead of half the strength getting added to the up we added it directionally only so you always jump x height but can
    // use more of the timing to aid in forward momentum.

    //remove any previous impulse on the object.
    impulse.clear();
    impulse.apply_impulse(Vec3::from((0.0, dynamic_jump_strength, 0.0)).into());

    info!(
        "\tJumped with {}/{} due to distance to ground, jump_factor {}, of ray length: {}",
        dynamic_jump_strength, JUMP_STRENGTH, jump_factor, ray_length
    );

    info!("\t ray_length {} ", ray_length);
}

/// Computes a clamped jump force factor based on the provided ray length.
///
/// # Arguments
///
/// * `ray_length` - The length of the ray used in the computation.
///
/// # Returns
///
/// The clamped jump force factor within the range [0.0, 1.0].
///
/// # Examples
///
/// ```
/// let ray_length = 3.0;
/// let jump_force_factor = compute_clamped_jump_force_factor(ray_length);
/// println!("Jump Force Factor: {}", jump_force_factor);
/// ```
fn compute_clamped_jump_force_factor(ray_length: f32) -> f32 {
    // Constants defined elsewhere in the code
    let full_standing_ray_length: f32 = RIDE_HEIGHT;
    let half_standing_ray_length: f32 = RIDE_HEIGHT - (CAPSULE_HEIGHT / 4.0);
    let standing_ray_length_range: f32 = full_standing_ray_length - half_standing_ray_length;

    // Ensure the input is within the specified range
    let clamped_ray_length = f32::clamp(ray_length, half_standing_ray_length, RIDE_HEIGHT);

    // Apply the linear transformation
    // Step 1: Normalize clamped_ray_length to a value between 0.0 and 1.0
    let normalized_distance =
        (clamped_ray_length - half_standing_ray_length) / standing_ray_length_range;

    // Step 2: Subtract the normalized distance from CAPSULE_HEIGHT
    let result: f32 = CAPSULE_HEIGHT - normalized_distance;

    // Ensure the output is within the range [0.0, 1.0]
    f32::clamp(result, 0.0, 1.0)
}
