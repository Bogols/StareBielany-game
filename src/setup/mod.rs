use crate::helpers::map_loader::{load_map_from_json, Map};
use crate::resources::constants::{TEXTURE_PATH, TILE_MAP_PATH};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
struct MyCameraMarker;

#[derive(Component)]
pub struct MainCamera;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 20.0)
            .with_scale(Vec3 {
                x: 10.,
                y: 10.,
                z: 10.,
            }),
        ..default()
    }, MainCamera));

    let map = load_map_from_json(TILE_MAP_PATH);
    let texture_handle = asset_server.load(TEXTURE_PATH);
    let texture_atlas = create_texture_atlas(texture_handle, &mut texture_atlases, &map);

    spawn_tiles(&mut commands, &map, &texture_atlas);
}

fn create_texture_atlas(
    texture_handle: Handle<Image>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    map: &Map,
) -> Handle<TextureAtlas> {
    let tileset = &map.tilesets[0];
    let tile_width = tileset.tilewidth;
    let tile_height = tileset.tileheight;
    let image_width = tileset.imagewidth;
    let image_height = tileset.imageheight;

    let cols = image_width / tile_width;
    let rows = image_height / tile_height;

    let mut texture_atlas = TextureAtlas::new_empty(
        texture_handle,
        Vec2::new(image_width as f32, image_height as f32),
    );

    for y in 0..rows {
        for x in 0..cols {
            let tile = Rect {
                min: Vec2::new(x as f32 * tile_width as f32, y as f32 * tile_height as f32),
                max: Vec2::new(
                    (x + 1) as f32 * tile_width as f32,
                    (y + 1) as f32 * tile_height as f32,
                ),
            };
            texture_atlas.add_texture(tile);
        }
    }

    texture_atlases.add(texture_atlas)
}

fn spawn_tiles(commands: &mut Commands, map: &Map, texture_atlas_handle: &Handle<TextureAtlas>) {
    let tile_width = map.tilewidth;
    let tile_height = map.tileheight;
    let tiles_in_row = map.width as usize;
    let tiles_in_col = map.height as usize;

    for (index, &tile_id) in map.layers[0].data.iter().enumerate() {
        if tile_id == 0 {
            continue;
        }

        let atlas_index = (tile_id - 1) as usize % (tiles_in_row * tiles_in_row);

        let x_pos = (index % tiles_in_row) as f32 * tile_width as f32;
        let y_pos = ((tiles_in_col - 1) - (index / tiles_in_row)) as f32 * tile_height as f32;

        commands.spawn(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(atlas_index),
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_xyz(x_pos, y_pos, 0.0),
            ..Default::default()
        });
    }
}
