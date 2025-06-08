use bevy_ecs_ldtk::prelude::*;

use super::animated_sprite::AnimationIndices;
use super::animated_sprite::AnimationTimer;
use super::movement::CharacterControllerBundle;
use crate::core::camera::SmoothFollow;
use crate::game::chain::CanAttachChain;
use crate::game::player_chain::CanShootChain;
use crate::prelude::*;
use crate::screen::Screen;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;

const PLAYER_WIDTH: f32 = 200.0;
const PLAYER_HEIGHT: f32 = 200.0;
/// The size of the sprite inside inkscape
const INKSCAPE_SCALE: f32 = 500.0;
const PLAYER_SCALE_X: f32 = PLAYER_WIDTH / INKSCAPE_SCALE;
const PLAYER_SCALE_Y: f32 = PLAYER_HEIGHT / INKSCAPE_SCALE;

// spritesheet indices
const IDLE_INDEX: usize = 0;
const RUN_FIRST_INDEX: usize = 1;
const RUN_LAST_INDEX: usize = 4;
const JUMP_INDEX: usize = 5;

// info for the eyes
const EYE_RADIUS: f32 = 25.0 * PLAYER_SCALE_X;
const LEFT_EYE_POS_X: f32 = convert_pos(263.0, PLAYER_SCALE_X);
const LEFT_EYE_POS_Y: f32 = convert_pos(157.0, -PLAYER_SCALE_Y);
const RIGHT_EYE_POS_X: f32 = convert_pos(385.0, PLAYER_SCALE_X);
const RIGHT_EYE_POS_Y: f32 = convert_pos(155.0, -PLAYER_SCALE_Y);

/// Convert between coords in inkscape to here (0, 0) being in top left vs in centre
const fn convert_pos(pos: f32, scale: f32) -> f32 {
    scale * (pos - INKSCAPE_SCALE / 2.0)
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<Direction>();

    app.add_event::<ChangePlayerDirection>();
    app.add_event::<ChangePlayerState>();

    app.configure::<PlayerAssets>();

    app.register_ldtk_entity::<PlayerBundle>("player");

    // app.add_systems(StateFlush, Screen::Gameplay.on_enter(spawn_player));
    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((
                process_player,
                set_camera_follow,
                change_player_direction,
                change_player_state,
                handle_eye_bobble,
            ))
            .in_set(PausableSystems),
    );
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[asset(path = "image/player_spritesheet.png")]
    player_spritesheet: Handle<Image>,
}

impl Configure for PlayerAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub enum PlayerState {
    #[default]
    Idle,
    Running,
    Jumping,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub enum Direction {
    Left,
    #[default]
    Right,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePlayerDirection {
    TurnLeft,
    TurnRight,
}

impl ChangePlayerDirection {
    fn as_direction(&self) -> Direction {
        match self {
            ChangePlayerDirection::TurnLeft => Direction::Left,
            ChangePlayerDirection::TurnRight => Direction::Right,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePlayerState {
    Running,
    Idle,
    Jumping,
}

impl ChangePlayerState {
    fn as_state(&self) -> PlayerState {
        match self {
            ChangePlayerState::Running => PlayerState::Running,
            ChangePlayerState::Idle => PlayerState::Idle,
            ChangePlayerState::Jumping => PlayerState::Jumping,
        }
    }

    fn as_indices(&self) -> AnimationIndices {
        match self {
            ChangePlayerState::Running => AnimationIndices::new(RUN_FIRST_INDEX, RUN_LAST_INDEX),
            ChangePlayerState::Idle => AnimationIndices::single(IDLE_INDEX),
            ChangePlayerState::Jumping => AnimationIndices::single(JUMP_INDEX),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum PlayerEye {
    Left,
    Right,
}

impl PlayerEye {
    fn get_pos(&self) -> Vec2 {
        match self {
            PlayerEye::Left => Vec2::new(LEFT_EYE_POS_X, LEFT_EYE_POS_Y),
            PlayerEye::Right => Vec2::new(RIGHT_EYE_POS_X, RIGHT_EYE_POS_Y),
        }
    }

    fn get_pos_with_dir(&self, direction: Direction) -> Vec2 {
        let pos = self.get_pos();
        if direction == Direction::Left {
            Vec2::new(-pos.x, pos.y)
        } else {
            Vec2::new(pos.x, pos.y)
        }
    }
}

#[derive(Bundle, Default, LdtkEntity)]
struct PlayerBundle {
    player: Player,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Deref, DerefMut, Reflect)]
#[reflect(Component)]
struct PlayerEyeBobble(pub Vec2);

fn process_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    new_player: Query<Entity, Added<Player>>,
    current_level: Res<LevelSelection>,
) {
    for player_entity in new_player.iter() {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(500), 6, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let mut player_sprite = Sprite::from_atlas_image(
            assets.player_spritesheet.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        );
        player_sprite.custom_size = Some((PLAYER_WIDTH, PLAYER_HEIGHT).into());

        let eye_mesh = meshes.add(Circle::new(EYE_RADIUS));
        let eye_material = materials.add(Color::BLACK);

        commands
            .entity(player_entity)
            .insert((
                player_sprite,
                PlayerState::default(),
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                AnimationIndices::single(IDLE_INDEX),
                Direction::default(),
                CharacterControllerBundle::new(Collider::ellipse(
                    PLAYER_WIDTH / 2.0,
                    PLAYER_HEIGHT / 2.0,
                )),
                Friction::new(0.1).with_combine_rule(CoefficientCombine::Min),
                Restitution::new(0.3).with_combine_rule(CoefficientCombine::Min),
                ColliderDensity(4.0),
                GravityScale(2.0),
                CollisionEventsEnabled,
                CanAttachChain,
            ))
            .insert_if(CanShootChain, || match *current_level {
                LevelSelection::Indices(level_indices) => level_indices.level > 0,
                _ => false,
            })
            .with_children(|player| {
                let eyes = [PlayerEye::Left, PlayerEye::Right];
                for eye in eyes {
                    let position = eye.get_pos().extend(1.0);

                    player.spawn((
                        Mesh2d(eye_mesh.clone()),
                        MeshMaterial2d(eye_material.clone()),
                        PlayerEyeBobble::default(),
                        eye,
                        Transform::from_translation(position),
                    ));
                }
            });
    }
}

fn set_camera_follow(
    mut camera: Single<&mut SmoothFollow, With<Camera2d>>,
    player_entity: Single<Entity, With<Player>>,
) {
    camera.target = *player_entity;
}

fn change_player_direction(
    mut player: Single<(&mut Direction, &mut Sprite, &Children), With<Player>>,
    mut player_eyes_query: Query<(&mut Transform, &PlayerEye)>,
    mut direction_event_reader: EventReader<ChangePlayerDirection>,
) {
    for event in direction_event_reader.read() {
        let existing_direction = *player.0;
        *player.0 = event.as_direction();
        player.1.flip_x = match event {
            ChangePlayerDirection::TurnLeft => true,
            ChangePlayerDirection::TurnRight => false,
        };

        for eye_entity in player.2.iter() {
            let player_eye = player_eyes_query.get_mut(eye_entity);
            if let Ok((mut player_eye_transform, player_eye)) = player_eye {
                let old_pos = player_eye.get_pos_with_dir(existing_direction);
                let existing_offset = player_eye_transform.translation.xy() - old_pos;
                let position = player_eye.get_pos_with_dir(event.as_direction()) + existing_offset;
                *player_eye_transform = Transform::from_translation(position.extend(1.0));
            }
        }
    }
}

fn handle_eye_bobble(
    time: Res<Time>,
    player_eye_bobble: Query<(&mut PlayerEyeBobble, &mut Transform, &PlayerEye)>,
    player_direction: Single<&Direction, With<Player>>,
) {
    const EYE_MOVEMENT_SPEED: f32 = 10.0;
    for (mut bobble, mut transform, eye_type) in player_eye_bobble {
        let eye_position = eye_type.get_pos_with_dir(**player_direction);
        let position_diff = eye_position - transform.translation.xy();
        let offset_diff = position_diff - **bobble;
        let normalised_offset = offset_diff.try_normalize();
        if let Some(normalised_offset) = normalised_offset {
            let offset_move = normalised_offset * time.delta_secs() * EYE_MOVEMENT_SPEED;
            if offset_move.length() >= offset_diff.length() {
                transform.translation += offset_diff.extend(0.0);
                **bobble = generate_random_eye_offset();
            } else {
                transform.translation += offset_move.extend(0.0);
            }
        } else {
            **bobble = generate_random_eye_offset();
        }
    }
}

fn generate_random_eye_offset() -> Vec2 {
    const EYE_X_OFFSET: f32 = 8.0;
    const EYE_Y_OFFSET: f32 = 16.0;

    let x = (random::<f32>() - 0.5) * EYE_X_OFFSET;
    let y = (random::<f32>() - 0.5) * EYE_Y_OFFSET;

    Vec2::new(x, y)
}

fn change_player_state(
    mut player: Single<(&mut PlayerState, &mut AnimationIndices), With<Player>>,
    mut state_event_reader: EventReader<ChangePlayerState>,
) {
    for event in state_event_reader.read() {
        *player.0 = event.as_state();
        *player.1 = event.as_indices();
    }
}
