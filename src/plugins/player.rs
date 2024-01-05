use benimator::FrameRate;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::resources::constants::PLAYER_SPEED;
use crate::setup::MainCamera;

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref, Clone, Debug)]
pub struct Animation(pub benimator::Animation);

#[derive(Default, Component, Deref, DerefMut)]
pub struct AnimationState(pub benimator::State);

#[derive(Component, Clone)]
pub struct PlayerAnimations {
    idle: Animation,
    running_right: Animation,
    running_left: Animation,
    running_down: Animation,
    running_up: Animation,
}

#[derive(Clone, Copy, Debug)]
enum PlayerAnimation {
    Idle,
    RunningLeft,
    RunningRight,
    RunningUp,
    RunningDown,
}

#[derive(Default, Resource)]
struct CursorPosition(Vec2);

#[derive(Component)]
struct Bullet {
    velocity: Vec2,
}

#[derive(Component)]
pub struct Pickup;

pub struct PlayerPlugin;

pub struct Health {
    current: i32,
    max: i32,
}

impl Health {
    fn new(max: i32) -> Self {
        Health { current: max, max }
    }
}

#[derive(Component)]
pub struct Enemy {
    health: Health,
}

impl Enemy {
    fn new(max_health: i32) -> Self {
        Enemy { health: Health::new(max_health) }
    }

    fn take_damage(&mut self, amount: i32) {
        self.health.current = (self.health.current - amount).max(0);
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(RapierConfiguration {
                gravity: Vec2::ZERO,
                ..default()
            })
            .insert_resource(CursorPosition(Vec2::ZERO))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
            .add_systems(Startup, player_setup)
            .add_systems(Startup, spawn_mob)
            .add_systems(Startup, spawn_wall)
            .add_systems(Startup, spawn_pickups)
            .add_systems(Update, set_cursor_position)
            .add_systems(Update, player_movement)
            .add_systems(Update, animate)
            .add_systems(Update, display_events)
            .add_systems(Update, print_entity_movement)
            .add_systems(Update, spawn_bullet_on_click)
            .add_systems(Update, move_bullets)
        ;
    }
}

pub fn set_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate()) {
        cursor_position.0 = world_position;
    }
}

fn display_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.read() {
        info!(
            "Received collision event: {:?}",
            collision_event
        );
    }

    for contact_force_event in contact_force_events.read() {
        info!(
            "Received contact force event: {:?}",
            contact_force_event
        );
    }
}

fn print_entity_movement(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput)>,
    mut commands: Commands,
    pickups: Query<&Pickup>,
    mut enemies: Query<(Entity, &mut Enemy)>,
) {
    for (entity, output) in controllers.iter() {
        if !output.collisions.is_empty() {
            for collision in &output.collisions {
                let collided_entity = collision.entity;

                if pickups.get(collided_entity).is_ok() {
                    info!("Entity {:?} collided with a Pickup", entity);
                    commands.entity(collided_entity).despawn();
                }

                if let Ok((enemy_entity, mut enemy)) = enemies.get_mut(collided_entity) {
                    enemy.take_damage(10);
                    info!("Entity {:?} collided with an Enemy. HP: {:?}/{:?}", entity, enemy.health.current, enemy.health.max);

                    if enemy.health.current == 0 {
                        commands.entity(enemy_entity).despawn();
                        info!("Enemy is dead.");
                    }
                }
            }

            // info!(
            //     "Entity {:?} moved by {:?} and touches the ground: {:?}, collisions: {:?}",
            //     entity,
            //     output.effective_translation,
            //     output.grounded,
            //     output.collisions
            // );
        }
    }
}

fn spawn_bullet_on_click(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    mut query: Query<&Transform, With<Player>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Ok(player_transform) = query.get_single_mut() {
            let player_position = player_transform.translation.truncate();
            let bullet_direction = cursor_position.0 - player_position; // Załóż, że masz `player_position`
            let bullet_velocity = bullet_direction.normalize() * 10000.0; // Przykładowa prędkość

            commands.spawn(Collider::capsule_y(50., 15.))
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(Bullet { velocity: bullet_velocity })
                .insert(TransformBundle::from(Transform::from_xyz(player_position.x, player_position.y, 0.)))
            ;
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

fn spawn_pickups(
    mut commands: Commands
) {
    let mut rng = thread_rng(); // Generator liczb losowych

    for _ in 0..10 {
        let x = rng.gen_range(-2000.0..2500.0); // Losowa pozycja X
        let y = rng.gen_range(-2000.0..2500.0); // Losowa pozycja Y

        commands
            .spawn((
                Transform::from_xyz(x, y, 0.),
                GlobalTransform::default(),
            ))
            .insert(Collider::capsule_y(200., 120.))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)
            .insert(Pickup)
            .insert(Name::new("pickup"));
    }
}

fn spawn_wall(
    mut commands: Commands
) {
    commands
        .spawn((
            TransformBundle::from(Transform::from_xyz(-1500., 100., 0.)),
            Collider::cuboid(50., 1500.),
            ActiveEvents::COLLISION_EVENTS
        ))
        .insert(Name::new("wall"))
    ;
}

fn spawn_mob(
    mut commands: Commands
) {
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(250.),
            ColliderMassProperties::Density(10.),
            TransformBundle::from(Transform::from_xyz(100., 100., 0.)),
            ActiveEvents::COLLISION_EVENTS
        ))
        .insert(Enemy::new(100))
        .insert(Name::new("mob"))
    ;
}

fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("wojtek-spritesheet.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(128.0, 128.0), 9, 5, None, None);
    let texture_atlas_handle = textures.add(texture_atlas);

    let player_animations = PlayerAnimations {
        idle: Animation(benimator::Animation::from_indices(
            0..=8,
            FrameRate::from_fps(10.0),
        )),
        running_right: Animation(benimator::Animation::from_indices(
            9..=17,
            FrameRate::from_fps(10.0),
        )),
        running_left: Animation(benimator::Animation::from_indices(
            18..=26,
            FrameRate::from_fps(10.0),
        )),
        running_down: Animation(benimator::Animation::from_indices(
            27..=35,
            FrameRate::from_fps(10.0),
        )),
        running_up: Animation(benimator::Animation::from_indices(
            36..=44,
            FrameRate::from_fps(10.0),
        )),
    };

    commands
        .spawn(RigidBody::KinematicPositionBased)
        .with_children(|children| {
            children.spawn((
                Collider::cuboid(20., 10.),
                TransformBundle::from(Transform::from_xyz(0.0, -60.0, 0.0)),
                Ccd::enabled(),
                KinematicCharacterController {
                    apply_impulse_to_dynamic_bodies: false,
                    ..default()
                },
                ActiveEvents::COLLISION_EVENTS
            ));
        })
        .insert(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(10.0)),
            ..Default::default()
        })
        .insert(player_animations.idle.clone())
        .insert(player_animations)
        .insert(AnimationState::default())
        .insert(Player)
        .insert(Name::new("player"))
    ;
}

pub fn player_movement(
    mut controllers: Query<&mut KinematicCharacterController>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationState,
            &mut Animation,
            &PlayerAnimations,
        ),
        With<Player>,
    >,
) {
    for mut controller in &mut controllers {
        for (mut state, mut animation, player_animations) in query.iter_mut() {
            let mut direction = Vec2::ZERO;
            let mut current_animation = PlayerAnimation::Idle;

            let directions = [
                (
                    KeyCode::W,
                    Vec2::new(0.0, 1.0),
                    PlayerAnimation::RunningUp,
                ),
                (
                    KeyCode::A,
                    Vec2::new(-1.0, 0.0),
                    PlayerAnimation::RunningLeft,
                ),
                (
                    KeyCode::S,
                    Vec2::new(0.0, -1.0),
                    PlayerAnimation::RunningDown,
                ),
                (
                    KeyCode::D,
                    Vec2::new(1.0, 0.0),
                    PlayerAnimation::RunningRight,
                ),
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
                PlayerAnimation::RunningLeft => &player_animations.running_left,
                PlayerAnimation::RunningRight => &player_animations.running_right,
                PlayerAnimation::RunningUp => &player_animations.running_up,
                PlayerAnimation::RunningDown => &player_animations.running_down,
            };

            if animation.0 != new_animation.0 {
                *animation = new_animation.clone();
            }

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
