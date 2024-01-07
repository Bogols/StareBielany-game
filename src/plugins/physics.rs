use bevy::math::Vec2;
use bevy::prelude::{App, default, Plugin};
use bevy_rapier2d::prelude::*;

const PIXELS_PER_METER: f32 = 100.;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(RapierConfiguration {
                gravity: Vec2::ZERO,
                ..default()
            })
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER))
            .add_plugins(RapierDebugRenderPlugin::default())
        ;
    }
}