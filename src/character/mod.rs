mod body;
mod focus;
mod motion;
pub mod stance;

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{io::memory::Dir, Assets},
    color::Color,
    ecs::event::ManualEventReader,
    hierarchy::{BuildChildren, Parent},
    input::mouse::MouseMotion,
    log::{info, warn},
    math::{Dir3, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    prelude::{
        apply_deferred, default, Bundle, Capsule3d, Commands, Component, Entity, IntoSystemConfigs,
        Query, ResMut, Resource, With, Without,
    },
    render::{camera::Camera, mesh::Mesh},
    transform::components::Transform,
};
use avian3d::prelude::*;

use body::Body;
use focus::{camera_look_system, Focus};
use motion::{update_player_motion, Motion};
use stance::{update_player_stance, Stance, StanceType};

use crate::{grab_cursor, CameraThing};

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing Character plugin...");
        app.insert_resource(Config::default()); // later we will load from some toml file
        app.add_systems(
            Startup,
            (spawn_player_system, attached_camera_system, grab_cursor).chain(),
        );
        app.add_systems(
            Update,
            (
                update_player_stance,
                update_player_motion,
                camera_look_system,
            ).chain(),
        );
        info!("Actor plugin successfully initialized!");
    }
}

#[derive(Resource)]
pub struct Config {
    capsule_height: f32,
    ride_height: f32,
    ray_length_offset: f32,
    ride_spring_strength: f32,
    ride_spring_damper: f32,
    stance_lockout: f32,
    jump_strength: f32,
    movement_speed: f32,
    sprint_speed_factor: f32,
    movement_decay: f32,
    look_sensitivity: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capsule_height: 1.0,
            ride_height: 1.5,
            ray_length_offset: 0.5,
            ride_spring_strength: 3500.0,
            ride_spring_damper: 300.0,
            stance_lockout: 0.25,
            jump_strength: 200.0,
            movement_speed: 50.0,
            sprint_speed_factor: 2.5,
            movement_decay: 0.90,
            look_sensitivity: 0.00012, // This value was taken from bevy_flycam.
        }
    }
}

pub trait GetDownwardRayLengthMax {
    fn get_downard_ray_length_max(&self) -> f32;
}

impl GetDownwardRayLengthMax for Config {
    fn get_downard_ray_length_max(&self) -> f32 {
        self.ride_height + self.ray_length_offset
    }
}

#[derive(Resource, Default)]
pub struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

// todo: create this one later
// #[derive(Bundle)]
// pub struct ActorBundle {
//     body: CharacterBody,
// }

#[derive(Component)]
pub struct PlayerControl;

#[derive(Bundle)]
pub struct PlayerBundle {
    linear_vel: LinearVelocity,
    external_force: ExternalForce,
    external_impulse: ExternalImpulse,
    rigid_body: RigidBody,
    mass: Mass,
    gravity_scale: GravityScale,
    collider: Collider,
    mat_mesh_bundle: MaterialMeshBundle<StandardMaterial>,
    downward_ray: RayCaster,
    ray_hits: RayHits,
    body: Body,
    motion: Motion,
    focus: Focus,
    stance: Stance,
}

fn spawn_player_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PlayerBundle {
            linear_vel: LinearVelocity::ZERO,
            external_force: ExternalForce::new([0.0, 0.0, 0.0].into()),
            external_impulse: ExternalImpulse::new([0.0, 0.0, 0.0].into()),
            rigid_body: RigidBody::Dynamic,
            mass: Mass(20.0),
            gravity_scale: GravityScale(1.0),
            collider: Collider::capsule(0.75, 0.5),
            mat_mesh_bundle: PbrBundle {
                mesh: meshes.add(Mesh::from(Capsule3d::new(0.5, 0.75))),
                material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
                transform: Transform::from_xyz(0.0, 32.0, 0.0),
                ..default()
            },
            downward_ray: RayCaster::new(Vec3::ZERO, Dir3::NEG_Y),
            ray_hits: RayHits::default(),
            body: Body { body_scale: 1.0 },
            motion: Motion {
                movement_vec: Vec3::from_array([0.0, 0.0, 0.0]),
                sprinting: false,
            },
            focus: Focus {
                point_of_focus: Vec3::from_array([0.0, 0.0, 0.0]),
                face_direction: Vec3::from_array([0.0, 0.0, 0.0]),
                free_look: false,
            },
            stance: Stance {
                current: StanceType::Standing,
                lockout: 0.0,
            },
        },
        PlayerControl,
    )); // it doesn't find thiss thing... maybe it needs to be in the inner objct
    info!("Spawned Player Actor");
}

fn attached_camera_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform), (With<PlayerControl>, Without<Camera>)>,
    mut camera_query: Query<
        (Entity, &mut Transform, Option<&Parent>),
        (With<Camera>, With<CameraThing>, Without<PlayerControl>),
    >,
) {
    if camera_query.is_empty()
        || camera_query.iter().len() > 1
        || player_query.is_empty()
        || player_query.iter().len() > 1
    {
        warn!("The camera attach system did not recieve 1 player and 1 camera. Found {} cameras, and {} players", camera_query.iter().len(), player_query.iter().len());
    }

    for (player_entity, _player_transform) in &mut player_query {
        for (camera_entity, mut camera_transform, camera_parent) in &mut camera_query {
            camera_transform.translation = Vec3::from_array([0.0, 1.0, 0.0]);
            if camera_parent.is_none() {
                commands
                    .entity(player_entity)
                    .push_children(&[camera_entity]);
                info!("Attached Camera to player character as child");
            } else {
                info!("Camera parent already exists, will not set player as parent! ");
            }
        }
    }
}
