use crate::resources::constants::{HALF_PLAYER_SIZE, PLAYER_SPEED};
use benimator::FrameRate;
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
pub struct Player;

#[derive(Component, Deref, Clone)]
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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, player_setup)
            .add_systems(Update, (player_movement, confine_player_movement))
            .add_systems(Update, animate);
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
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(10.0)),
            ..Default::default()
        })
        .insert(player_animations)
        .insert(AnimationState::default())
        .insert(Player {});
}

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &mut AnimationState,
            &mut Animation,
            &PlayerAnimations,
        ),
        With<Player>,
    >,
) {
    if let Ok((mut transform, mut state, mut animation, player_animations)) = query.get_single_mut()
    {
        let mut direction = Vec3::ZERO;
        let mut current_animation = PlayerAnimation::Idle;

        let directions = [
            (
                KeyCode::W,
                Vec3::new(0.0, 1.0, 0.0),
                PlayerAnimation::RunningUp,
            ),
            (
                KeyCode::A,
                Vec3::new(-1.0, 0.0, 0.0),
                PlayerAnimation::RunningLeft,
            ),
            (
                KeyCode::S,
                Vec3::new(0.0, -1.0, 0.0),
                PlayerAnimation::RunningDown,
            ),
            (
                KeyCode::D,
                Vec3::new(1.0, 0.0, 0.0),
                PlayerAnimation::RunningRight,
            ),
        ];

        for (key, vec, anim) in directions.iter() {
            println!("key: {:?}, vec: {:?}, anim: {:?}", key, vec, anim);

            if keyboard_input.pressed(*key) {
                direction += *vec;
                current_animation = *anim;
            }
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
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

pub fn confine_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();
        let x_min = HALF_PLAYER_SIZE;
        let x_max = window.width() - HALF_PLAYER_SIZE;
        let y_min = HALF_PLAYER_SIZE;
        let y_max = window.height() - HALF_PLAYER_SIZE;

        player_transform.translation.x = player_transform.translation.x.clamp(x_min, x_max);
        player_transform.translation.y = player_transform.translation.y.clamp(y_min, y_max);
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
