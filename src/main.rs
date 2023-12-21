use bevy::prelude::*;
use setup::setup;
use systems::{player_movement, confine_player_movement};

mod components;
mod systems;
mod setup;
mod resources;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, confine_player_movement))
        .run();
}
