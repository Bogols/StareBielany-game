use bevy::prelude::Component;

use crate::components::health::Health;

#[derive(Component)]
pub struct Enemy {
    pub(crate) health: Health,
    pub(crate) speed: f32,
    pub(crate) player_spotted: bool,
}

impl Enemy {
    pub(crate) fn new(max_health: i32, speed: f32) -> Self {
        Enemy {
            health: Health::new(max_health),
            speed,
            player_spotted: false,
        }
    }

    pub(crate) fn take_damage(&mut self, amount: i32) {
        self.health.current = (self.health.current - amount).max(0);
    }
}