use bevy::app::App;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{Commands, Component, ParamSet, Plugin, Query, Res, Startup, Time, Timer, TimerMode, Transform, TransformBundle, Update, With};
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{ActiveEvents, Collider, ColliderMassProperties};
use rand::{random, Rng, thread_rng};

use crate::components::enemy::Enemy;
use crate::components::player::Player;

#[derive(Component)]
pub struct EnemyTimer(pub(crate) Timer);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_enemy)
            .add_systems(Update, move_enemies)
            .add_systems(Update, chase_player)
        ;
    }
}

fn spawn_enemy(
    mut commands: Commands
) {
    let mut rng = thread_rng();

    for _ in 0..10 {
        let x = rng.gen_range(-200.0..250.0);
        let y = rng.gen_range(-200.0..250.0);

        commands
            .spawn((
                RigidBody::Dynamic,
                Collider::ball(25.),
                ColliderMassProperties::Mass(100.),
                TransformBundle::from(Transform::from_xyz(x, y, 0.)),
                ActiveEvents::COLLISION_EVENTS
            ))
            .insert(Enemy::new(100, 100.0))
            .insert(EnemyTimer(Timer::from_seconds(1.0, TimerMode::Once)))
        ;
    }
}

fn chase_player(
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&mut Enemy, &mut Transform)>
    )>,
) {
    let player_position = query_set.p0().get_single().map(|t| t.translation);

    if let Ok(player_translation) = player_position {
        for (mut enemy, mut enemy_transform) in query_set.p1().iter_mut() {
            let direction_to_player = player_translation - enemy_transform.translation;
            let distance_to_player = direction_to_player.length();

            if distance_to_player < 250.0 {
                enemy.player_spotted = true;
                let direction_normalized = direction_to_player.normalize_or_zero();
                enemy_transform.translation += direction_normalized * time.delta_seconds() * enemy.speed;
            } else {
                enemy.player_spotted = false;
            }
        }
    }
}

fn move_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Enemy, &mut EnemyTimer)>,
) {
    for (mut transform, enemy, mut timer) in query.iter_mut() {
        if enemy.player_spotted == false {
            return;
        }

        timer.0.tick(time.delta());
        if timer.0.finished() {
            let random_angle = random::<f32>() * 2.0 * std::f32::consts::PI;
            transform.rotation = Quat::from_rotation_z(random_angle);
            timer.0.reset();
        }

        let forward = transform.rotation * Vec3::Y;
        transform.translation += forward * enemy.speed * time.delta_seconds();
    }
}
