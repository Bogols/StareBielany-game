use crate::helpers::tiled::TiledMapPlugin;
use bevy::prelude::*;
use setup::setup;
use systems::{confine_player_movement, player_movement};

mod components;
mod helpers;
mod resources;
mod setup;
mod systems;

fn main() {
    println!("{:?}", std::env::current_dir());
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TiledMapPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, confine_player_movement))
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
