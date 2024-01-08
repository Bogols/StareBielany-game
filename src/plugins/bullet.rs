use bevy::app::{App, Update};
use bevy::log::info;
use bevy::prelude::{Commands, Entity, EventReader, Plugin, Query, Res, Resource, Time, Timer, Transform, With};
use bevy::time::TimerMode;
use bevy_rapier2d::pipeline::CollisionEvent;

use crate::components::bullet::Bullet;
use crate::components::enemy::Enemy;
use crate::components::wall::Wall;

#[derive(Default, Resource)]
pub struct BulletSpawnTimer(pub(crate) Timer);

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BulletSpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
            .add_systems(Update, destroy_expired_bullets)
            .add_systems(Update, move_bullets)
            .add_systems(Update, listen_collision_events)
        ;
    }
}

fn listen_collision_events(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    bullet_query: Query<Entity, With<Bullet>>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    wall_query: Query<Entity, With<Wall>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(entity1, entity2, _) => {
                handle_bullet_collision(
                    &mut commands,
                    entity1,
                    entity2,
                    &bullet_query,
                    &mut enemy_query,
                    &wall_query,
                );
            }
            _ => {}
        }

        // info!("Received collision event: {:?}", collision_event);
    }
}

pub fn handle_bullet_collision(
    commands: &mut Commands,
    entity1: &Entity,
    entity2: &Entity,
    bullet_query: &Query<Entity, With<Bullet>>,
    enemy_query: &mut Query<(Entity, &mut Enemy)>,
    wall_query: &Query<Entity, With<Wall>>,
) {
    if let Ok(bullet_entity) = get_bullet_entity(entity1, entity2, bullet_query) {
        if process_bullet_enemy_collision(bullet_entity, entity1, entity2, commands, enemy_query)
            || process_bullet_wall_collision(bullet_entity, entity1, entity2, commands, wall_query) {
            return;
        }
    }

    // info!("No relevant bullet collision detected.");
}

fn get_bullet_entity<'a>(
    entity1: &'a Entity,
    entity2: &'a Entity,
    bullet_query: &'a Query<Entity, With<Bullet>>,
) -> Result<&'a Entity, ()> {
    if bullet_query.get(*entity1).is_ok() {
        Ok(entity1)
    } else if bullet_query.get(*entity2).is_ok() {
        Ok(entity2)
    } else {
        Err(())
    }
}

fn process_bullet_enemy_collision(
    bullet_entity: &Entity,
    entity1: &Entity,
    entity2: &Entity,
    commands: &mut Commands,
    enemy_query: &mut Query<(Entity, &mut Enemy)>,
) -> bool {
    let (bullet, enemy) = if enemy_query.get_mut(*entity1).is_ok() {
        (bullet_entity, entity1)
    } else if enemy_query.get_mut(*entity2).is_ok() {
        (bullet_entity, entity2)
    } else {
        return false;
    };

    if let Ok((enemy_entity, mut enemy)) = enemy_query.get_mut(*enemy) {
        enemy.take_damage(10);
        commands.entity(*bullet).despawn();
        // info!("Enemy hit! Remaining HP: {}", enemy.health.current);

        if enemy.health.current == 0 {
            commands.entity(enemy_entity).despawn();
            // info!("Enemy is dead.");
        }
        true
    } else {
        false
    }
}

fn process_bullet_wall_collision(
    bullet_entity: &Entity,
    entity1: &Entity,
    entity2: &Entity,
    commands: &mut Commands,
    wall_query: &Query<Entity, With<Wall>>,
) -> bool {
    if wall_query.get(*entity1).is_ok() || wall_query.get(*entity2).is_ok() {
        commands.entity(*bullet_entity).despawn();
        // info!("Bullet despawned upon hitting a wall.");
        true
    } else {
        false
    }
}

fn destroy_expired_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut bullet) in query.iter_mut() {
        if bullet.lifetime.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn move_bullets(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Bullet)>,
) {
    for (mut transform, bullet) in query.iter_mut() {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_seconds();
    }
}
