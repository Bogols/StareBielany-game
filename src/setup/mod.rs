use crate::components::player::Player;
use crate::helpers;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
struct MyCameraMarker;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 20.0),
        ..default()
    });

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("brand64.png"),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        },
        Player {},
    ));

    // let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("test6.tmx");
    // println!("{:?}", map_handle);
    //
    // commands.spawn(helpers::tiled::TiledMapBundle {
    //     tiled_map: map_handle,
    //     ..Default::default()
    // });
    let teture_handle = asset_server.load("metro-tiles.png");
    let mut texture_atlas =
        TextureAtlas::new_empty(teture_handle, Vec2::new(tilemap_width, tilemap_height));

    for y in 0..number_of_tiles_high {
        for x in 0..number_of_tiles_wide {
            let tile = Rect {
                min: Vec2::new(x as f32 * 32.0, y as f32 * 32.0),
                max: Vec2::new((x + 1) as f32 * 32.0, (y + 1) as f32 * 32.0),
            };
            texture_atlas.add_texture(tile);
        }
    }

    let texture_atlas_handle = texture_atlases.add(texture_atlas);
}
