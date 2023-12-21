use crate::helpers::tiled::TiledMapPlugin;
use bevy::prelude::*;
use setup::setup;
use systems::{confine_player_movement, player_movement};

mod components;
mod helpers;
mod resources;
mod setup;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(TiledMapPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, confine_player_movement))
        .run();
}
