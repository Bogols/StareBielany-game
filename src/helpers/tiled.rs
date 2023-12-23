use std::io::{Cursor, ErrorKind};
use std::path::Path;
use std::sync::Arc;

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt},
    log,
    prelude::{
        Added, Asset, AssetApp, AssetEvent, AssetId, Assets, Bundle, Commands, Component,
        DespawnRecursiveExt, Entity, EventReader, GlobalTransform, Handle, Image, Plugin, Query,
        Res, Transform, Update,
    },
    reflect::TypePath,
    utils::{BoxedFuture, HashMap},
};
use bevy_ecs_tilemap::prelude::*;

use thiserror::Error;

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<TiledMap>()
            .register_asset_loader(TiledLoader)
            .add_systems(Update, process_loaded_maps);
    }
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,

    pub tilemap_textures: HashMap<usize, TilemapTexture>,

    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}

#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> std::result::Result<Self::Resource, Self::Error> {
        Ok(Cursor::new(self.bytes.clone()))
    }
}

pub struct TiledLoader;

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    #[error("Could not load Tiled file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut loader = tiled::Loader::with_cache_and_reader(
                tiled::DefaultResourceCache::new(),
                BytesResourceReader::new(&bytes),
            );
            let map = loader.load_tmx_map(load_context.path()).map_err(|e| {
                std::io::Error::new(ErrorKind::Other, format!("Could not load TMX map: {e}"))
            })?;

            let mut dependencies = Vec::new();
            let mut tilemap_textures = HashMap::default();
            let mut tile_image_offsets = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                if let Some(img) = &tileset.image {
                    let tmx_dir = load_context
                        .path()
                        .parent()
                        .expect("The asset load context was empty.");
                    let tile_path = tmx_dir.join(&img.source);
                    let asset_path = AssetPath::from(tile_path);
                    let texture: Handle<Image> = load_context.load(asset_path.clone());
                    dependencies.push(asset_path);

                    tilemap_textures.insert(tileset_index, TilemapTexture::Single(texture.clone()));
                } else {
                    let mut tile_images: Vec<Handle<Image>> = Vec::new();
                    for (tile_id, tile) in tileset.tiles() {
                        if let Some(img) = &tile.image {
                            let tmx_dir = load_context
                                .path()
                                .parent()
                                .expect("The asset load context was empty.");
                            let tile_path = tmx_dir.join(&img.source);
                            let asset_path = AssetPath::from(tile_path);
                            log::info!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                            let texture: Handle<Image> = load_context.load(asset_path.clone());
                            tile_image_offsets
                                .insert((tileset_index, tile_id), tile_images.len() as u32);
                            tile_images.push(texture.clone());
                            dependencies.push(asset_path);
                        }
                    }

                    tilemap_textures.insert(tileset_index, TilemapTexture::Vector(tile_images));
                }
            }

            let asset_map = TiledMap {
                map,
                tilemap_textures,
                tile_image_offsets,
            };

            log::info!("Loaded map: {}", load_context.path().display());
            Ok(asset_map)
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(&Handle<TiledMap>, &mut TiledLayersStorage)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    let mut changed_maps = Vec::<AssetId<TiledMap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                log::info!("Map added!");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                log::info!("Map changed!");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.id());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle.id() != *changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                // TODO: Create a RemoveMap component..
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn_recursive()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index)
                    else {
                        log::warn!("Skipped creating layer with missing tilemap textures.");
                        continue;
                    };

                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() else {
                            log::info!(
                                "Skipping layer {} because only tile layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let tiled::TileLayer::Finite(layer_data) = tile_layer else {
                            log::info!(
                                "Skipping layer {} because only finite layers are supported.",
                                layer.id()
                            );
                            continue;
                        };

                        let map_size = TilemapSize {
                            x: tiled_map.map.width,
                            y: tiled_map.map.height,
                        };

                        let grid_size = TilemapGridSize {
                            x: tiled_map.map.tile_width as f32,
                            y: tiled_map.map.tile_height as f32,
                        };

                        let map_type = match tiled_map.map.orientation {
                            tiled::Orientation::Hexagonal => {
                                TilemapType::Hexagon(HexCoordSystem::Row)
                            }
                            tiled::Orientation::Isometric => {
                                TilemapType::Isometric(IsoCoordSystem::Diamond)
                            }
                            tiled::Orientation::Staggered => {
                                TilemapType::Isometric(IsoCoordSystem::Staggered)
                            }
                            tiled::Orientation::Orthogonal => TilemapType::Square,
                        };

                        let mut tile_storage = TileStorage::empty(map_size);
                        let layer_entity = commands.spawn_empty().id();

                        for x in 0..map_size.x {
                            for y in 0..map_size.y {
                                // Transform TMX coords into bevy coords.
                                let mapped_y = tiled_map.map.height - 1 - y;

                                let mapped_x = x as i32;
                                let mapped_y = mapped_y as i32;

                                let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                    Some(t) => t,
                                    None => {
                                        continue;
                                    }
                                };
                                if tileset_index != layer_tile.tileset_index() {
                                    continue;
                                }
                                let layer_tile_data =
                                    match layer_data.get_tile_data(mapped_x, mapped_y) {
                                        Some(d) => d,
                                        None => {
                                            continue;
                                        }
                                    };
                                let texture_index = match tilemap_texture {
                                        TilemapTexture::Single(_) => layer_tile.id(),
                                        TilemapTexture::Vector(_) => {
                                            *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile.id()))
                                                .expect("The offset into the image vector should have been saved during the initial load.")
                                        },
                                        TilemapTexture::TextureContainer(_) => {
                                            // Handle the TextureContainer case here.
                                            // You will need to decide how to get the texture_index based on your use case.
                                            // For example, if TextureContainer behaves similarly to Vector, you might need a similar approach.
                                            // Or if it's a different logic, implement that here.
                                            unimplemented!("TextureContainer handling not implemented")
                                        }
                                        // Add any other variants here with appropriate handling.
                                    };

                                let tile_pos = TilePos { x, y };
                                let tile_entity = commands
                                    .spawn(TileBundle {
                                        color: TileColor(bevy::render::color::Color::Rgba {
                                            // debug purposes
                                            red: 225.00,
                                            green: 0.00,
                                            blue: 0.00,
                                            alpha: 1.00,
                                        }),
                                        position: tile_pos,
                                        tilemap_id: TilemapId(layer_entity),
                                        texture_index: TileTextureIndex(texture_index),
                                        flip: TileFlip {
                                            x: layer_tile_data.flip_h,
                                            y: layer_tile_data.flip_v,
                                            d: layer_tile_data.flip_d,
                                        },
                                        ..Default::default()
                                    })
                                    .id();
                                tile_storage.set(&tile_pos, tile_entity);
                            }
                        }

                        commands.entity(layer_entity).insert(TilemapBundle {
                            grid_size,
                            size: map_size,
                            storage: tile_storage,
                            texture: tilemap_texture.clone(),
                            tile_size,
                            spacing: tile_spacing,
                            transform: {
                                let transform = get_tilemap_center_transform(
                                    &map_size,
                                    &grid_size,
                                    &map_type,
                                    layer_index as f32,
                                ) * Transform::from_xyz(offset_x, -offset_y, 0.0);

                                println!("Transform: {:?}", transform);
                                println!("map size: {:?}", &map_size);

                                transform
                            },
                            ..Default::default()
                        });

                        layer_storage
                            .storage
                            .insert(layer_index as u32, layer_entity);
                    }
                }
            }
        }
    }
}
