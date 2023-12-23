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
    columns: u32,
    firstgid: u32,
    image: String,
    imageheight: u32,
    imagewidth: u32,
    margin: u32,
    name: String,
    spacing: u32,
    tilecount: u32,
    tileheight: u32,
    tilewidth: u32,
}

pub fn load_map_from_json(path: &str) -> Map {
    let file_content = std::fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&file_content).expect("Unable to parse json")
}
