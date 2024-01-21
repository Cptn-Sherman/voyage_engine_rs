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
    log::info,
    math::{Ray, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    render::{
        camera::Camera,
        color::Color,
        mesh::{shape, Mesh},
        texture::Image,
    },
    transform::components::Transform,
    utils::default,
};
use bevy_xpbd_3d::{
    components::{Collider, ExternalForce, LinearVelocity, Mass, RigidBody, GravityScale},
    parry::{query::RayCast, simba::scalar::SupersetOf},
    plugins::spatial_query::{RayCaster, RayHits, ShapeCaster},
};

pub struct FirstPersonPlayerControllerPlugin;

impl Plugin for FirstPersonPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing player controller plugin");
        app.add_systems(Startup, spawn_player_system);
        app.add_systems(Update, update_player_data_system);
    }
}

const RIDE_HEIGHT: f32 = 1.5;
const DOWNWARD_RAY_LENGTH_MAX: f32 = 1.0 + RIDE_HEIGHT;
const RIDE_SPRING_STRENGTH: f32 = 800.0;
const RIDE_SPRING_DAMPER: f32 = 75.0;

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
pub struct PlayerData;

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
            collider: Collider::capsule(1.0, 0.5), // <--- these values may be wrong.
            mat_mesh_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(0.0, 8.0, 0.0),
                ..default()
            },
            data: PlayerData,
        },
        RayCaster::new(Vec3::ZERO, Vec3::NEG_Y),
    ));
}

fn update_player_data_system(
    mut query: Query<(
        &RayCaster,
        &RayHits,
        &LinearVelocity,
        &mut GravityScale,
        &mut ExternalForce,
        &mut PlayerData,
    )>,
) {
    for (ray, hits, mut vel, mut gravity, mut external_force, data) in &mut query {
        if hits.is_empty() {
            gravity.0 = 1.0;
            info!("downward raycast did not hit anything");
            continue;
        } else if let Some(hit) = hits.iter_sorted().next() {
            // Get the length of the ray to the first position we hit.
            let ray_distance: f32 = Vec3::length(ray.direction * hit.time_of_impact);
            // if this ray is longer than the DOWNWARD_RAY_LENGTH_MAX disregard.
            if f32::abs(ray_distance) > DOWNWARD_RAY_LENGTH_MAX {
                gravity.0 = 1.0;
                continue;
            } else {
                info!("Gravity off");
                gravity.0 = 0.0;
            }

            let spring_offset = f32::abs(ray_distance) - RIDE_HEIGHT;
            let spring_force = (spring_offset * RIDE_SPRING_STRENGTH) - (-vel.y * RIDE_SPRING_DAMPER);

            info!(
                "ray_direction {}, ray distance {}, spring_offset {}, spring_force {}",
                ray.direction, ray_distance, spring_offset, spring_force
            );

            // apply the spring force to the player controller rigid body
            external_force.clear();
            external_force.apply_force(Vec3::from((0.0, -spring_force, 0.0)));
            
        }
    }
}
