use std::{num, ops::Sub};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    log::info,
    math::{Ray, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    render::{
        camera::Camera,
        color::Color,
        mesh::{shape, Mesh},
        texture::Image,
    },
    time::Time,
    transform::components::Transform,
    utils::default,
};
use bevy_xpbd_3d::{
    components::{
        Collider, ExternalForce, ExternalImpulse, GravityScale, LinearVelocity, Mass, RigidBody,
    },
    parry::{query::RayCast, simba::scalar::SupersetOf},
    plugins::spatial_query::{RayCaster, RayHits, ShapeCaster},
};

use crate::EngineSettings;

const CAPSULE_HEIGHT: f32 = 1.0;
const RIDE_HEIGHT: f32 = 1.5;
const RAY_LENGTH_OFFSET: f32 = 0.5;
const DOWNWARD_RAY_LENGTH_MAX: f32 = RAY_LENGTH_OFFSET + RIDE_HEIGHT;
const RIDE_SPRING_STRENGTH: f32 = 800.0;
const RIDE_SPRING_DAMPER: f32 = 75.0;
const DEFAULT_STANCE_LOCKOUT: f32 = 0.25;
const JUMP_STRENGTH: f32 = 80.0;

pub struct FirstPersonPlayerControllerPlugin;

impl Plugin for FirstPersonPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing player controller plugin");
        app.add_systems(Startup, spawn_player_system);
        app.add_systems(Update, update_player_data_system);
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
    previous_jump_factor: f32,
}

#[derive(Debug, PartialEq, Clone)]
enum PlayerStance {
    Standing,
    Landing,
    Jumping,
    Falling,
}

fn spawn_player_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("spawning player contorller");
    commands.spawn((
        PlayerBundle {
            rigid_body: RigidBody::Dynamic,
            mass: Mass(10.0),
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
                current_stance: PlayerStance::Falling,
                stance_lockout: 0.0,
                previous_jump_factor: 0.0,
            },
        },
        RayCaster::new(Vec3::ZERO, Vec3::NEG_Y),
    ));
}

fn update_player_data_system(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
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
    for (ray, hits, mut vel, mut gravity, mut external_force, mut external_impulse, mut data) in
        &mut query
    {
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
            PlayerStance::Falling => {
                // Set the gravity scale to zero.
                data.gravity_scale = 1.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();
            }
            PlayerStance::Jumping => {
                // Set the gravity scale to zero.
                data.gravity_scale = 1.0;

                // Clear any persisting forces on the rigid body.
                external_force.clear();

                if data.current_stance != PlayerStance::Jumping {
                    // we have to clear the velocity when we jump.
                    vel.y = 0.0;

                    apply_jump_force(&mut data, &mut external_impulse, ray_length, time.delta_seconds());
                }
            }
        }

        // Update the gravity scale.
        gravity.0 = data.gravity_scale;

        // Update the current stance.
        data.current_stance = next_stance.clone();
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
            next_stance = PlayerStance::Falling;
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
            next_stance = PlayerStance::Falling;
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
    // Compute this value by subtracting the ray length from the set ride height to find the diference in position.
    let spring_offset = f32::abs(ray_length) - RIDE_HEIGHT;
    let spring_force = (spring_offset * RIDE_SPRING_STRENGTH) - (-velocity_y * RIDE_SPRING_DAMPER);

    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
    force.clear();
    force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
}

fn apply_jump_force(data: &mut PlayerData, impulse: &mut ExternalImpulse, ray_length: f32, delta_time: f32) {
    // Apply the stance cooldown now that we are jumping
    data.stance_lockout = DEFAULT_STANCE_LOCKOUT;

    let one_quarter_jump_strength: f32 = JUMP_STRENGTH / 4.0;
    let three_quarter_jump_strength: f32 = one_quarter_jump_strength * 3.0;
    let jump_factor = compute_clamped_jump_force_factor(ray_length, data.previous_jump_factor);

    // update the previous value


    data.previous_jump_factor = jump_factor - delta_time;


    let dynamic_jump_strength: f32 =
        one_quarter_jump_strength + (three_quarter_jump_strength * jump_factor);

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
fn compute_clamped_jump_force_factor(ray_length: f32, previous_val: f32) -> f32 {
    // Constants defined elsewhere in the code
    let full_standing_ray_length: f32 = RIDE_HEIGHT;
    let half_standing_ray_length = RIDE_HEIGHT - (CAPSULE_HEIGHT / 4.0);
    let standing_ray_length_range: f32 = full_standing_ray_length - half_standing_ray_length;

    // Ensure the input is within the specified range
    let clamped_ray_length = f32::clamp(ray_length, half_standing_ray_length, RIDE_HEIGHT);

    // Apply the linear transformation

    // Step 1: Normalize clamped_ray_length to a value between 0.0 and 1.0
    let normalized_distance = (clamped_ray_length - half_standing_ray_length) / standing_ray_length_range;

    // Step 2: Subtract the normalized distance from CAPSULE_HEIGHT
    let result: f32 = CAPSULE_HEIGHT - normalized_distance;

    // Ensure the output is within the range [0.0, 1.0]
    let clamped_result = f32::clamp(result, 0.0, 1.0);

    let final_result = f32::max(clamped_result, previous_val / 2.0);

    // Return the final result
    final_result
}
