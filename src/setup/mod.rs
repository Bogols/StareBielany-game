use crate::components::player::Player;
use crate::helpers::map_loader::{load_map_from_json, Map};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
struct MyCameraMarker;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
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

    let map = load_map_from_json("assets/test12.json");

    let texture_handle = asset_server.load("metro-tiles.png");
    let texture_atlas = create_texture_atlas(
        texture_handle,
        &mut texture_atlases,
        map.tilewidth,
        map.tileheight,
        map.width,
        map.height,
    );

    spawn_tiles(&mut commands, &map, &texture_atlas);
}

fn create_texture_atlas(
    texture_handle: Handle<Image>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    tile_width: u32,
    tile_height: u32,
    map_width: u32,
    map_height: u32,
) -> Handle<TextureAtlas> {
    let tile_size = tile_width as f32;
    let tilemap_width = map_width * tile_width;
    let tilemap_height = map_height * tile_height;

    let mut texture_atlas = TextureAtlas::new_empty(
        texture_handle,
        Vec2::new(tilemap_width as f32, tilemap_height as f32),
    );

    let cols = map_width;
    let rows = map_height;

    for y in 0..rows as i32 {
        for x in 0..cols as i32 {
            let tile = Rect {
                min: Vec2::new(x as f32 * tile_size, y as f32 * tile_size),
                max: Vec2::new((x + 1) as f32 * tile_size, (y + 1) as f32 * tile_size),
            };
            texture_atlas.add_texture(tile);
        }
    }

    texture_atlases.add(texture_atlas)
}

fn spawn_tiles(commands: &mut Commands, map: &Map, texture_atlas_handle: &Handle<TextureAtlas>) {
    let mut z_index = 0.0;
    let tile_size = map.tilewidth as f32;
    let tiles_in_row = map.width as usize;

    println!("tiles_in_row: {}", tiles_in_row);

    for layer in &map.layers {
        for (index, &tile_id) in layer.data.iter().enumerate() {
            if tile_id == 0 {
                continue;
            }

            let tile_texture_index = tile_id - 1;

            let x_pos = (index % tiles_in_row) as f32 * tile_size;
            let y_pos = (index / tiles_in_row) as f32 * tile_size;

            println!("x_pos: {}, y_pos: {}", x_pos, y_pos);

            commands.spawn(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(tile_texture_index as usize),
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_xyz(x_pos, y_pos, z_index),
                ..Default::default()
            });
        }
        z_index += 1.0;
    }
}
