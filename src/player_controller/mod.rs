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

pub struct FirstPersonPlayerControllerPlugin;

impl Plugin for FirstPersonPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing player controller plugin");
        app.add_systems(Startup, spawn_player_system);
        app.add_systems(Update, update_player_data_system);
    }
}

const GRAVITY_SCALE_SPEED_FACTOR: f32 = 1.0;

const RIDE_HEIGHT: f32 = 1.5;
const RAY_LENGTH_OFFSET: f32 = 0.5;
const DOWNWARD_RAY_LENGTH_MAX: f32 = RAY_LENGTH_OFFSET + RIDE_HEIGHT;
const RIDE_SPRING_STRENGTH: f32 = 800.0;
const RIDE_SPRING_DAMPER: f32 = 75.0;

const DEFAULT_STANCE_LOCKOUT: f32 = 0.25;

const JUMP_STRENGTH: f32 = 80.0;

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
}
#[derive(Debug, PartialEq, Clone)]
enum PlayerStance {
    Standing,
    Landing,
    Jumping,
    Crouched,
    Prone,
    Seated,
    Laying,
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
            collider: Collider::capsule(1.0, 0.5),
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
            },
        },
        RayCaster::new(Vec3::ZERO, Vec3::NEG_Y),
    ));
}

fn determine_stance(
    keys: &Res<Input<KeyCode>>,
    ray: &RayCaster,
    hits: &RayHits,
    data: &PlayerData,
) -> PlayerStance {
    
    let is_locked_out: bool = data.stance_lockout > 0.0;
    let previous_stance: PlayerStance = data.current_stance.clone();
    let mut next_stance: PlayerStance = data.current_stance.clone();

    // If your locked in you cannot change state.
    if !is_locked_out {
        if let Some(hit) = hits.iter_sorted().next() {
            // Get the length of the ray to the first position we hit.
            let ray_distance: f32 = Vec3::length(ray.direction * hit.time_of_impact);
            let abs_ray_distance: f32 = f32::abs(ray_distance);

            if abs_ray_distance < RIDE_HEIGHT {
                next_stance = PlayerStance::Standing;
            } else if previous_stance != PlayerStance::Standing
                && abs_ray_distance < DOWNWARD_RAY_LENGTH_MAX
            {
                next_stance = PlayerStance::Landing;
            } else if abs_ray_distance > DOWNWARD_RAY_LENGTH_MAX {
                next_stance = PlayerStance::Falling;
            }
        } else {
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
        // what was I doing last frame, update that state.

        // We update the state of the jump_cooldown
        data.stance_lockout -= time.delta_seconds();
        data.stance_lockout = f32::clamp(data.stance_lockout, 0.0, 1.0);

        let mut ray_distance: f32 = 0.0;
        if let Some(hit) = hits.iter_sorted().next() {
            ray_distance = Vec3::length(ray.direction * hit.time_of_impact);
        }

        let mut next_stance: PlayerStance = determine_stance(&keys, ray, hits, &data);

        match next_stance {
            PlayerStance::Landing => {
                // Set the gravity scale to zero.
                data.gravity_scale = 0.0;

                // --- STANDING: APPLY STANDING SPRING FORCE ---

                // Find the diference between how close the capsule is to the surface beneath it.
                // Compute this value by subtracting the ray length from the set ride height to find the diference in position.
                let spring_offset = f32::abs(ray_distance) - RIDE_HEIGHT;
                let spring_force =
                    (spring_offset * RIDE_SPRING_STRENGTH) - (-vel.y * RIDE_SPRING_DAMPER);

                /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
                external_force.clear();
                external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
            }
            PlayerStance::Standing => {
                // Set the gravity scale to zero.
                data.gravity_scale = 0.0;

                // Check to see if the player is jumping.
                if data.stance_lockout <= 0.0 && keys.pressed(KeyCode::X) {
                    // we have to clear the velocity when we jump.
                    vel.y = 0.0;

                    // Apply the jump cooldown now that we are jumping
                    data.stance_lockout = DEFAULT_STANCE_LOCKOUT;
                    next_stance = PlayerStance::Jumping;

                    // This calculation is not very accurate, ideally we would use the amount the downward ray extends past RIDE_HEIGHT subtracted from the total possible difference.
                    // todo: Allowing a jump from the absolute bottom to be the full jump strength. BUT I dont want to spend more time on that so ... later.
                    let inverse_spring_offset_factor = DOWNWARD_RAY_LENGTH_MAX - f32::abs(ray_distance);
                    let half_default_jump_strength = JUMP_STRENGTH / 2.0;
                    let scaled_jump_strength: f32 = half_default_jump_strength
                        + (half_default_jump_strength * inverse_spring_offset_factor);

                    //remove any previous impulse on the object.
                    external_impulse.clear();
                    external_force.clear();
                    external_impulse
                        .apply_impulse(Vec3::from((0.0, scaled_jump_strength, 0.0)).into());

                    info!(
                        "Detected Stance Change:{:#?} -> {:#?}, cleared velocity, forces, and impulse.",
                        data.current_stance, next_stance
                    );

                    info!(
                        "\tJumped with {}/{} due to distance to ground",
                        scaled_jump_strength, JUMP_STRENGTH
                    );
                } else {
                    // Clear any persisting forces on the rigid body.
                    external_force.clear();

                    // todo: This Apply Standing Spring Force needs to be broken into a method however, the logic breaks when passing the external_force outside this function.
                    // --- STANDING: APPLY STANDING SPRING FORCE ---

                    // Find the diference between how close the capsule is to the surface beneath it.
                    // Compute this value by subtracting the ray length from the set ride height to find the diference in position.
                    let spring_offset = f32::abs(ray_distance) - RIDE_HEIGHT;
                    let spring_force =
                        (spring_offset * RIDE_SPRING_STRENGTH) - (-vel.y * RIDE_SPRING_DAMPER);

                    /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
                    external_force.clear();
                    external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
                }
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
            }
            PlayerStance::Crouched => todo!(),
            PlayerStance::Prone => todo!(),
            PlayerStance::Seated => todo!(),
            PlayerStance::Laying => todo!(),
        }

        gravity.0 = data.gravity_scale;
        data.current_stance = next_stance.clone();

        // info!(
        //     "Stance: {:#?}, lockout {}, gravity_scale {}",
        //     next_stance, data.stance_lockout, data.gravity_scale
        // )
    }
}

fn apply_spring_force(ray_distance: f32, velocity_y: f32, mut force: ExternalForce) {}
