use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: f32 = 64.0; // This is the player sprite size.
pub const HALF_PLAYER_SIZE: f32 = PLAYER_SIZE / 2.00;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .add_systems(Update, confine_player_movement)
        .run();
}

#[derive(Component)]
pub struct Player {}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("brand64.png"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Player {}
    ));
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut sprite_position_query: Query<&mut Transform, With<Player>>
) {
    if let Ok(mut transform) = sprite_position_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        let directions = [
            (KeyCode::W, Vec3::new(0.0, 1.0, 0.0)),
            (KeyCode::A, Vec3::new(-1.0, 0.0, 0.0)),
            (KeyCode::S, Vec3::new(0.0, -1.0, 0.0)),
            (KeyCode::D, Vec3::new(1.0, 0.0, 0.0)),
        ];

        for (key, vec) in directions.iter() {
            if keyboard_input.pressed(*key) {
                direction += *vec;
            }
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        let x_min = HALF_PLAYER_SIZE;
        let x_max = window.width() - HALF_PLAYER_SIZE;
        let y_min = HALF_PLAYER_SIZE;
        let y_max = window.height() - HALF_PLAYER_SIZE;

        player_transform.translation.x = player_transform.translation.x.clamp(x_min, x_max);
        player_transform.translation.y = player_transform.translation.y.clamp(y_min, y_max);
    }
}

