use benimator::FrameRate;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::components::bullet::Bullet;
use crate::components::pickup::Pickup;
use crate::components::player::Player;
use crate::plugins::bullet::BulletSpawnTimer;
use crate::plugins::cursor_position::CursorPosition;
use crate::resources::constants::PLAYER_SPEED;

#[derive(Component, Deref, Clone, Debug)]
pub struct Animation(pub benimator::Animation);

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

#[derive(Component, Clone)]
pub struct PlayerAnimations {
    idle: Animation,
    running: Animation,
}

#[derive(Clone, Copy, Debug)]
enum PlayerAnimation {
    Idle,
    Running,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, player_setup)
            .add_systems(Update, player_movement)
            .add_systems(Update, animate)
            .add_systems(Update, listen_player_controller)
            .add_systems(Update, spawn_bullets_on_pressed);
    }
}

fn listen_player_controller(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput)>,
    mut commands: Commands,
    pickups: Query<&Pickup>,
) {
    for (_entity, output) in controllers.iter() {
        if !output.collisions.is_empty() {
            for collision in &output.collisions {
                let collided_entity = collision.entity;

                if pickups.get(collided_entity).is_ok() {
                    commands.entity(collided_entity).despawn();
                }
            }
        }
    }
}

fn spawn_bullets_on_pressed(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    query: Query<&Transform, With<Player>>,
    mut bullet_spawn_timer: ResMut<BulletSpawnTimer>,
    time: Res<Time>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        if bullet_spawn_timer.0.tick(time.delta()).just_finished() {
            if let Ok(player_transform) = query.get_single() {
                let player_position = player_transform.translation.truncate();
                let bullet_direction = cursor_position.0 - player_position;
                let bullet_velocity = bullet_direction.normalize_or_zero() * 1000.0;
                let bullet_angle =
                    bullet_direction.y.atan2(bullet_direction.x) + std::f32::consts::FRAC_PI_2;

                commands
                    .spawn(Collider::capsule_y(5., 1.5))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(Bullet {
                        velocity: bullet_velocity,
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    })
                    .insert(TransformBundle::from(
                        Transform::from_xyz(player_position.x, player_position.y, 0.)
                            .with_rotation(Quat::from_rotation_z(bullet_angle)),
                    ));
            }
        }
    }
}

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("wojtek-spritesheet-v3.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(96.0, 96.0), 9, 2, None, None);
    let texture_atlas_handle = textures.add(texture_atlas);

    let player_animations = PlayerAnimations {
        idle: Animation(benimator::Animation::from_indices(
            0..=8,
            FrameRate::from_fps(10.0),
        )),
        running: Animation(benimator::Animation::from_indices(
            9..=17,
            FrameRate::from_fps(10.0),
        )),
    };

    commands
        .spawn(RigidBody::KinematicPositionBased)
        .with_children(|children| {
            children.spawn((
                Collider::ball(20.),
                TransformBundle::from(Transform::from_xyz(0., 0., 0.)),
                Ccd::enabled(),
                KinematicCharacterController {
                    apply_impulse_to_dynamic_bodies: false,
                    ..default()
                },
                ActiveEvents::COLLISION_EVENTS,
            ));
        })
        .insert(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..Default::default()
        })
        .insert(player_animations.idle.clone())
        .insert(player_animations)
        .insert(AnimationState::default())
        .insert(Player);
}

fn player_movement(
    mut controllers: Query<&mut KinematicCharacterController>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<
        (
            &mut AnimationState,
            &mut Animation,
            &PlayerAnimations,
            &mut Transform,
        ),
        With<Player>,
    >,
    cursor_position: Res<CursorPosition>,
) {
    for mut controller in &mut controllers {
        for (mut state, mut animation, player_animations, mut transform) in player_query.iter_mut()
        {
            let mut direction = Vec2::ZERO;
            let mut current_animation = PlayerAnimation::Idle;

            let directions = [
                (KeyCode::W, Vec2::new(0.0, 1.0), PlayerAnimation::Running),
                (KeyCode::A, Vec2::new(-1.0, 0.0), PlayerAnimation::Running),
                (KeyCode::S, Vec2::new(0.0, -1.0), PlayerAnimation::Running),
                (KeyCode::D, Vec2::new(1.0, 0.0), PlayerAnimation::Running),
            ];

            for (key, vec, anim) in directions.iter() {
                if keyboard_input.pressed(*key) {
                    direction += *vec;
                    current_animation = *anim;
                }
            }

            if direction != Vec2::ZERO {
                direction = direction.normalize();
                controller.translation = Some(direction * PLAYER_SPEED * time.delta_seconds());
            } else {
                current_animation = PlayerAnimation::Idle;
            }

            let new_animation = match current_animation {
                PlayerAnimation::Idle => &player_animations.idle,
                PlayerAnimation::Running => &player_animations.running,
            };

            if animation.0 != new_animation.0 {
                *animation = new_animation.clone();
            }

            let player_position_vec = transform.translation.truncate();
            let player_direction_vec = cursor_position.0 - player_position_vec;
            let angle = player_direction_vec.y.atan2(player_direction_vec.x) + std::f32::consts::PI;

            info!(angle);

            transform.rotation = Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2);

            state.update(&animation.0, time.delta());
        }
    }
}
fn animate(
    time: Res<Time>,
    mut query: Query<(&mut AnimationState, &mut TextureAtlasSprite, &Animation)>,
) {
    for (mut player, mut texture, animation) in query.iter_mut() {
        player.update(&animation.0, time.delta());

        texture.index = player.frame_index();
    }
}
