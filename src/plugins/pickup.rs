use bevy::prelude::{App, Commands, GlobalTransform, Plugin, Startup, Transform};
use bevy_rapier2d::geometry::{ActiveEvents, Collider, Sensor};
use rand::{Rng, thread_rng};

use crate::components::pickup::Pickup;

pub struct PickupPlugin;

impl Plugin for PickupPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_sample_pickups)
        ;
    }
}

fn spawn_sample_pickups(
    mut commands: Commands
) {
    let mut rng = thread_rng();

    for _ in 0..10 {
        let x = rng.gen_range(-200.0..250.0);
        let y = rng.gen_range(-200.0..250.0);

        commands
            .spawn((
                Transform::from_xyz(x, y, 0.),
                GlobalTransform::default(),
            ))
            .insert(Collider::capsule_y(2., 1.2))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)
            .insert(Pickup)
        ;
    }
}
