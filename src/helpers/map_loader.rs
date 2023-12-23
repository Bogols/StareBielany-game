use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub height: u32,
    pub width: u32,
    pub layers: Vec<Layer>,
    pub infinite: bool,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub tilesets: Vec<Tileset>,
    pub version: String,
    pub orientation: String,
    pub renderorder: String,
}

#[derive(Serialize, Deserialize)]

pub struct Layer {
    pub data: Vec<u32>,
    height: u32,
    width: u32,
    name: String,
    opacity: f32,
    visible: bool,
    x: u32,
    y: u32,
}

#[derive(Serialize, Deserialize)]

pub struct Tileset {
    pub columns: u32,
    pub firstgid: u32,
    pub image: String,
    pub imageheight: u32,
    pub imagewidth: u32,
    pub margin: u32,
    pub name: String,
    pub spacing: u32,
    pub tilecount: u32,
    pub tileheight: u32,
    pub tilewidth: u32,
}

pub fn load_map_from_json(path: &str) -> Map {
    let file_content = std::fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&file_content).expect("Unable to parse json")
}
