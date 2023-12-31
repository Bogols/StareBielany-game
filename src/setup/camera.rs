use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct MainCamera;

pub fn camera_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.),
            ..default()
        },
        MainCamera,
    ));

    let map_texture_handle = asset_server.load("1_map_stare-bielany-v3.png");

    commands.spawn(SpriteBundle {
        texture: map_texture_handle,
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0)
            .with_scale(Vec3 {
                x: (2.),
                y: (2.),
                z: (0.),
            }),
        ..Default::default()
    });
}
