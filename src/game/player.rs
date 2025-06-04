use super::{
    animated_sprite::{AnimationIndices, AnimationTimer},
    movement::CharacterControllerBundle,
};
use crate::{core::camera::SmoothFollow, game::chain::CanAttachChain, prelude::*, screen::Screen};
use bevy_ecs_ldtk::prelude::*;

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
const RUN_LAST_INDEX: usize = 3;
const JUMP_INDEX: usize = 4;

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
    app.add_systems(Update, Screen::Gameplay.on_update(process_player));
    app.add_systems(Update, Screen::Gameplay.on_update(set_camera_follow));
    app.add_systems(Update, Screen::Gameplay.on_update(change_player_direction));
    app.add_systems(Update, Screen::Gameplay.on_update(change_player_state));
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
    fn to_direction(&self) -> Direction {
        match self {
            ChangePlayerDirection::TurnLeft => Direction::Left,
            ChangePlayerDirection::TurnRight => Direction::Right,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangePlayerState {
    ChangeModeRunning,
    ChangeModeIdle,
    ChangeModeJumping,
}

impl ChangePlayerState {
    fn to_state(&self) -> PlayerState {
        match self {
            ChangePlayerState::ChangeModeRunning => PlayerState::Running,
            ChangePlayerState::ChangeModeIdle => PlayerState::Idle,
            ChangePlayerState::ChangeModeJumping => PlayerState::Jumping,
        }
    }

    fn to_indices(&self) -> AnimationIndices {
        match self {
            ChangePlayerState::ChangeModeRunning => {
                AnimationIndices::new(RUN_FIRST_INDEX, RUN_LAST_INDEX)
            },
            ChangePlayerState::ChangeModeIdle => AnimationIndices::single(IDLE_INDEX),
            ChangePlayerState::ChangeModeJumping => AnimationIndices::single(JUMP_INDEX),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum PlayerEye {
    Left,
    Right,
}

#[derive(Bundle, Default, LdtkEntity)]
struct PlayerBundle {
    player: Player,
}

fn process_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    new_player: Query<Entity, Added<Player>>,
) {
    for player_entity in new_player.iter() {
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(500), 5, 1, None, None);
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
            .with_children(|player| {
                player.spawn((
                    Mesh2d(eye_mesh.clone()),
                    MeshMaterial2d(eye_material.clone()),
                    PlayerEye::Left,
                    Transform::from_xyz(LEFT_EYE_POS_X, LEFT_EYE_POS_Y, 1.0),
                ));
                player.spawn((
                    Mesh2d(eye_mesh.clone()),
                    MeshMaterial2d(eye_material.clone()),
                    PlayerEye::Right,
                    Transform::from_xyz(RIGHT_EYE_POS_X, RIGHT_EYE_POS_Y, 1.0),
                ));
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
        *player.0 = event.to_direction();
        player.1.flip_x = match event {
            ChangePlayerDirection::TurnLeft => true,
            ChangePlayerDirection::TurnRight => false,
        };

        for eye_entity in player.2.iter() {
            let player_eye = player_eyes_query.get_mut(eye_entity);
            if let Ok(mut player_eye) = player_eye {
                *player_eye.0 = match (event, player_eye.1) {
                    (ChangePlayerDirection::TurnLeft, PlayerEye::Left) => {
                        Transform::from_xyz(-LEFT_EYE_POS_X, LEFT_EYE_POS_Y, 1.0)
                    },
                    (ChangePlayerDirection::TurnRight, PlayerEye::Left) => {
                        Transform::from_xyz(LEFT_EYE_POS_X, LEFT_EYE_POS_Y, 1.0)
                    },
                    (ChangePlayerDirection::TurnLeft, PlayerEye::Right) => {
                        Transform::from_xyz(-RIGHT_EYE_POS_X, RIGHT_EYE_POS_Y, 1.0)
                    },
                    (ChangePlayerDirection::TurnRight, PlayerEye::Right) => {
                        Transform::from_xyz(RIGHT_EYE_POS_X, RIGHT_EYE_POS_Y, 1.0)
                    },
                };
            }
        }
    }
}

fn change_player_state(
    mut player: Single<(&mut PlayerState, &mut AnimationIndices), With<Player>>,
    mut state_event_reader: EventReader<ChangePlayerState>,
) {
    for event in state_event_reader.read() {
        *player.0 = event.to_state();
        *player.1 = event.to_indices();
    }
}
