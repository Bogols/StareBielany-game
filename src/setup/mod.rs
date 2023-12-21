use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::components::player::Player;


pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>, window_query: Query<&Window, With<PrimaryWindow>>) {
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