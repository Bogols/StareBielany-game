use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::components::player::Player;
use crate::resources::constants::HALF_PLAYER_SIZE;

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
