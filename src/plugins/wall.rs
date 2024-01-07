use bevy::app::{App, Plugin, Startup};
use bevy::core::Name;
use bevy::prelude::{Commands, Transform, TransformBundle};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{ActiveEvents, Collider, Sensor};

use crate::components::wall::Wall;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_wall)
        ;
    }
}

fn spawn_wall(
    mut commands: Commands
) {
    /*
    * @TODO: Wall a bit hacked by rapier collisions, but working properly?
    *   It's not. Enemy has an ability to "walk" through the wall.
    */
    commands
        .spawn((
            TransformBundle::from(Transform::from_xyz(-150., 10., 0.)),
            Collider::cuboid(50., 1500.),
            RigidBody::Dynamic,
            ActiveEvents::COLLISION_EVENTS,
            Sensor
        ))
        .insert(Wall)
        .insert(Name::new("wall"))
    ;
}
