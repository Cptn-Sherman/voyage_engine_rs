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

const GRAVITY_SCALE_SPEED_FACTOR: f32 = 2.0;

const RIDE_HEIGHT: f32 = 1.5;
const DOWNWARD_RAY_LENGTH_MAX: f32 = 1.0 + RIDE_HEIGHT;
const RIDE_SPRING_STRENGTH: f32 = 800.0;
const RIDE_SPRING_DAMPER: f32 = 75.0;

const JUMP_STRENGTH: f32 = 180.0;
const JUMP_COOLDOWN_LENGTH: f32 = 0.65;

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
    jump_cooldown: f32,
    gravity_scale: f32,
    is_grounded: bool,
    current_stance: PlayerStance,
    target_stance: PlayerStance,
}

enum PlayerStance {
    Standing,
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
                jump_cooldown: 0.0,
                gravity_scale: 1.0,
                is_grounded: false,
                current_stance: PlayerStance::Falling,
                target_stance: PlayerStance::Falling,
            },
        },
        RayCaster::new(Vec3::ZERO, Vec3::NEG_Y),
    ));
}

fn update_player_data_system(
    time: Res<Time>,
    engine_settings: Res<EngineSettings>,
    keys: Res<Input<KeyCode>>,
    mut query: Query<(
        &RayCaster,
        &RayHits,
        &LinearVelocity,
        &mut GravityScale,
        &mut ExternalForce,
        &mut ExternalImpulse,
        &mut PlayerData,
    )>,
) {
    for (ray, hits, vel, mut gravity, mut external_force, mut external_impulse, mut data) in
        &mut query
    {
        // what was I doing last frame, update that state.
        // We update the state of the jump_cooldown
        data.jump_cooldown -= time.delta_seconds();
        data.jump_cooldown = f32::clamp(data.jump_cooldown, 0.0, 1.0);

        data.stance = determine_stance(); // write a function which determines which stance we are in. or break this into two parts,
        // the system which determines which stance a entity is based on these collision checks.
        // Followed by a 

        match &data.stance {
            PlayerStance::Standing => {

            },
            PlayerStance::Falling => {},
            PlayerStance::Jumping => {},
        }

        // what am I doing this frame, Standing, Jumping, or Falling.

        // Have I become grounded this frame?
        if let Some(hit) = hits.iter_sorted().next() {
            // Get the length of the ray to the first position we hit.
            let ray_distance: f32 = Vec3::length(ray.direction * hit.time_of_impact);
            // if this ray is longer than the DOWNWARD_RAY_LENGTH_MAX we do not apply the spring force.
            if f32::abs(ray_distance) > DOWNWARD_RAY_LENGTH_MAX {
                // -- Falling --
                data.is_grounded = false;

                // Progress the jump cooldown and set is_grounded to false.
                data.jump_cooldown -= time.delta_seconds();

                // Increase the impact of gravity to zero over time based on GRAVITY_SCALE_SPEED_FACTOR, Capped at 1.0.
                data.gravity_scale += GRAVITY_SCALE_SPEED_FACTOR * time.delta_seconds();
                data.gravity_scale = f32::clamp(data.gravity_scale, 0.0, 1.0);

                // info!(
                //     "jump_cooldown {}, is_grounded {}",
                //     data.jump_cooldown, data.is_grounded
                // );
            } else {
                // --- STANDING ---

                if !data.is_grounded {
                    // JUST GROUNDED THIS TICK DO LANDING SHIT.
                    info!("Standing!");
                }
                data.is_grounded = true;

                data.jump_cooldown -= time.delta_seconds();

                // Reduce the impact of gravity to zero over time based on GRAVITY_SCALE_SPEED_FACTOR, Capped at 0.0.
                data.gravity_scale -= GRAVITY_SCALE_SPEED_FACTOR * time.delta_seconds();
                data.gravity_scale = f32::clamp(data.gravity_scale, 0.0, 1.0);

                // --- STANDING: CHECK FOR JUMP ACTION ---

                if data.jump_cooldown <= 0.0 && keys.pressed(KeyCode::X) {
                    data.is_grounded = false;
                    data.jump_cooldown = JUMP_COOLDOWN_LENGTH;

                    // remove any previous impulse on the object.
                    external_impulse.clear();
                    external_impulse.apply_impulse(Vec3::from((0.0, JUMP_STRENGTH, 0.0)).into());
                    info!("Jumping!");
                    continue;
                }

                // --- STANDING: APPLY SPRING FORCE ---

                // Find the diference between how close the capsule is to the surface beneath it.
                // Compute this value by subtracting the ray length from the set ride height to find the diference in position.
                let spring_offset = f32::abs(ray_distance) - RIDE_HEIGHT;
                let spring_force =
                    (spring_offset * RIDE_SPRING_STRENGTH) - (-vel.y * RIDE_SPRING_DAMPER);

                /* Now we apply our spring force vector in the direction to return the bodies distance from the ground towards RIDE_HEIGHT. */
                external_force.clear();
                external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));

                /* -- DEBUG OUTPUT FOR STANDING SPRING FORCE --
                info!(
                    "ray_direction {},
                    ray distance {},
                    spring_offset {},
                    spring_force {}",
                    ray.direction,
                    ray_distance,
                    spring_offset,
                    spring_force
                );
                */
            }

            // info!(
            //     "jump_cooldown {}, is_grounded {}",
            //     data.jump_cooldown, data.is_grounded
            // );
        }
    }
}
