use bevy::prelude::*;
use crate::components::player::Player;
use crate::resources::constants::PLAYER_SPEED;


pub fn player_movement(
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

