use bevy::prelude::*;

use plugins::physics::PhysicsPlugin;
use plugins::pickup::PickupPlugin;
use plugins::player::PlayerPlugin;
use setup::camera::camera_setup;

use crate::plugins::bullet::BulletPlugin;
use crate::plugins::cursor_position::CursorPositionPlugin;
use crate::plugins::enemy::EnemyPlugin;
use crate::plugins::wall::WallPlugin;

mod components;
mod helpers;
mod plugins;
mod resources;
mod setup;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, camera_setup)
        .add_plugins(PhysicsPlugin)
        .add_plugins(CursorPositionPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(BulletPlugin)
        .add_plugins(WallPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(PickupPlugin)
        .add_systems(Update, camera_movement_system)
        // .add_systems(Update, print_camera_position)
        .run();
}

fn camera_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            direction.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Up) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            direction.y -= 1.0;
        }

        let speed = 2.0;
        transform.translation += direction * speed;
    }
}

fn print_camera_position(query: Query<&Transform, With<Camera>>) {
    for transform in query.iter() {
        println!(
            "Camera Position: x = {}, y = {}",
            transform.translation.x, transform.translation.y
        );
    }
}
