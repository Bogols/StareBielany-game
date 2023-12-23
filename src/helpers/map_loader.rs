use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Map {
    height: u32,
    width: u32,
    layers: Vec<Layer>,
    infinite: bool,
    tileheight: u32,
    tilewidth: u32,
    tilesets: Vec<Tileset>,
    version: String,
    orientation: String,
    renderorder: String,
}

#[derive(Serialize, Deserialize)]

struct Layer {
    data: Vec<u32>,
    height: u32,
    width: u32,
    name: String,
    opacity: f32,
    visible: bool,
    x: u32,
    y: u32,
}

#[derive(Serialize, Deserialize)]

struct Tileset {
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

fn load_map_from_json(path: &str) -> Map {
    let file_content = std::fs::read_to_string(path).expect("Unable to read file");
    serde_json::from_str(&file_content).expect("Unable to parse json")
}
