use benimator::FrameRate;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;
use bevy::ecs::system::ParamSet;

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
    lifetime: Timer,
}

#[derive(Default, Resource)]
struct BulletSpawnTimer(Timer);

#[derive(Component)]
pub struct Pickup;

#[derive(Component)]
pub struct Wall;

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
    speed: f32,
}

#[derive(Component)]
struct EnemyTimer(Timer);

impl Enemy {
    fn new(max_health: i32, speed: f32) -> Self {
        Enemy {
            health: Health::new(max_health),
            speed,
        }
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
            .insert_resource(BulletSpawnTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
            .add_systems(Startup, player_setup)
            .add_systems(Startup, spawn_enemy)
            .add_systems(Startup, spawn_wall)
            .add_systems(Startup, spawn_pickups)
            .add_systems(Update, set_cursor_position)
            .add_systems(Update, player_movement)
            .add_systems(Update, animate)
            .add_systems(Update, display_events)
            .add_systems(Update, print_entity_movement)
            .add_systems(Update, spawn_bullet_on_click)
            .add_systems(Update, despawn_expired_bullets)
            .add_systems(Update, move_bullets)
            .add_systems(Update, move_enemies)
            .add_systems(Update, chase_player)
        ;
    }
}

fn set_cursor_position(
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

        info!("Received collision event: {:?}", collision_event);
    }
}

fn handle_bullet_collision(
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
    info!("No relevant bullet collision detected.");
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

    // Odebranie obrażeń przez przeciwnika i usunięcie pocisku
    if let Ok((enemy_entity, mut enemy)) = enemy_query.get_mut(*enemy) {
        enemy.take_damage(10);
        commands.entity(*bullet).despawn();
        info!("Enemy hit! Remaining HP: {}", enemy.health.current);

        if enemy.health.current == 0 {
            commands.entity(enemy_entity).despawn();
            info!("Enemy is dead.");
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
        info!("Bullet despawned upon hitting a wall.");
        true
    } else {
        false
    }
}

fn print_entity_movement(
    controllers: Query<(Entity, &KinematicCharacterControllerOutput)>,
    mut commands: Commands,
    pickups: Query<&Pickup>,
) {
    for (entity, output) in controllers.iter() {
        if !output.collisions.is_empty() {
            for collision in &output.collisions {
                let collided_entity = collision.entity;

                if pickups.get(collided_entity).is_ok() {
                    info!("Entity {:?} collided with a Pickup", entity);
                    commands.entity(collided_entity).despawn();
                }
            }
        }
    }
}

fn spawn_bullet_on_click(
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
                let bullet_velocity = bullet_direction.normalize_or_zero() * 10000.0;
                let bullet_angle = bullet_direction.y.atan2(bullet_direction.x) + std::f32::consts::FRAC_PI_2;

                commands.spawn(Collider::capsule_y(50., 15.))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(Bullet {
                        velocity: bullet_velocity,
                        lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                    })
                    .insert(TransformBundle::from(Transform::from_xyz(player_position.x, player_position.y, 0.)
                        .with_rotation(Quat::from_rotation_z(bullet_angle))
                    ));
            }
        }
    }
}

fn despawn_expired_bullets(
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
    /*
    * @TODO: Wall a bit hacked by rapier collisions, but working properly?
    *   It's not. Enemy has an ability to "walk" through the wall.
    */
    commands
        .spawn((
            TransformBundle::from(Transform::from_xyz(-1500., 100., 0.)),
            Collider::cuboid(50., 1500.),
            RigidBody::Dynamic,
            ActiveEvents::COLLISION_EVENTS,
            Sensor
        ))
        .insert(Wall)
        .insert(Name::new("wall"))
    ;
}

fn spawn_enemy(
    mut commands: Commands
) {
    let mut rng = thread_rng(); // Generator liczb losowych

    for _ in 0..10 {
        let x = rng.gen_range(-2000.0..2500.0); // Losowa pozycja X
        let y = rng.gen_range(-2000.0..2500.0); // Losowa pozycja Y

        commands
            .spawn((
                RigidBody::Dynamic,
                Collider::ball(250.),
                TransformBundle::from(Transform::from_xyz(x, y, 0.)),
                ActiveEvents::COLLISION_EVENTS
            ))
            .insert(Enemy::new(100, 1500.0))
            .insert(EnemyTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        ;
    }
}

fn chase_player(
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<&Transform, With<Player>>,
        Query<(&Enemy, &mut Transform)>
    )>,
) {
    // Najpierw zbieramy informacje o położeniu gracza
    let player_position = query_set.p0().get_single().map(|t| t.translation);

    // Następnie iterujemy przez przeciwników
    if let Ok(player_translation) = player_position {
        for (enemy, mut enemy_transform) in query_set.p1().iter_mut() {
            let direction_to_player = player_translation - enemy_transform.translation;
            let distance_to_player = direction_to_player.length();

            if distance_to_player < 5000.0 {
                let direction_normalized = direction_to_player.normalize_or_zero();
                enemy_transform.translation += direction_normalized * time.delta_seconds() * enemy.speed;
            }
        }
    }
}

fn move_enemies(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Enemy, &mut EnemyTimer)>,
) {
    for (mut transform, enemy, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            // Losowe zmienienie kierunku
            let random_angle = random::<f32>() * 2.0 * std::f32::consts::PI;
            transform.rotation = Quat::from_rotation_z(random_angle);
            timer.0.reset();
        }

        // Poruszanie się w kierunku ustalonym przez rotację
        let forward = transform.rotation * Vec3::Y;
        transform.translation += forward * enemy.speed * time.delta_seconds();
    }
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
