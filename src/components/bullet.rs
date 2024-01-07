use bevy::math::Vec2;
use bevy::prelude::{Component, Timer};

#[derive(Component)]
pub struct Bullet {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: Timer,
}
