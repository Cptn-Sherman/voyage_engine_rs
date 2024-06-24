use bevy::{pbr::{MaterialMeshBundle, StandardMaterial}, prelude::Component};
use bevy_xpbd_3d::{components::{Collider, GravityScale, Mass, RigidBody}, prelude::RayCaster};

#[derive(Component)]
pub struct Body {
    pub body_scale: f32,
    pub rigid_body: RigidBody,
    pub mass: Mass,
    pub gravity_scale: GravityScale,
    pub collider: Collider,
    pub mat_mesh_bundle: MaterialMeshBundle<StandardMaterial>,
    pub downward_ray: RayCaster,
}