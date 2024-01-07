use bevy::app::App;
use bevy::math::Vec2;
use bevy::prelude::{Camera, GlobalTransform, Plugin, Query, ResMut, Resource, Update, Window, With};
use bevy::window::PrimaryWindow;

use crate::setup::camera::MainCamera;

#[derive(Default, Resource)]
pub struct CursorPosition(pub(crate) Vec2);

pub struct CursorPositionPlugin;

impl Plugin for CursorPositionPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CursorPosition(Vec2::ZERO))
            .add_systems(Update, set_cursor_position)
        ;
    }
}

fn set_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate()) {
        cursor_position.0 = world_position;
    }
}