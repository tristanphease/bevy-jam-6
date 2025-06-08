use crate::{
    game::{
        animated_sprite::{AnimationIndices, AnimationTimer},
        goal::Goal,
    },
    menu::Menu,
    prelude::*,
    screen::{Screen, gameplay::ShowGameOverMenu},
};

const FIRE_SPREAD_OFFSET: f32 = 40.0;
const FIRE_INDICES: usize = 4;
const MAX_FIRE_INDEX: usize = 100;

pub(super) fn plugin(app: &mut App) {
    app.configure::<EndSequence>();
    app.configure::<EndSequenceAssets>();
    app.configure::<EndSequenceSystems>();
    app.configure::<EndSequencePausedSystems>();

    app.add_event::<StartEndSequenceEvent>();

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update(start_end_sequence)
            .in_set(PausableSystems)
            .in_set(EndSequencePausedSystems),
    );
    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update(process_fire_spread)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update(handle_end_timer)
            .in_set(EndSequenceSystems),
    );

    app.add_systems(StateFlush, Menu::GameOver.on_exit(reset_end_sequence));
}

#[derive(Event, Reflect, Copy, Clone, Eq, PartialEq, Debug)]
pub struct StartEndSequenceEvent;

#[derive(State, Reflect, Copy, Clone, Default, Eq, PartialEq, Debug)]
#[state(log_flush)]
#[reflect(Resource)]
pub struct EndSequence;

impl Configure for EndSequence {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.add_state::<Self>();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpreadDirection {
    Left,
    Right,
    Up,
    Down,
}

impl SpreadDirection {
    fn all() -> Vec<Self> {
        vec![
            SpreadDirection::Left,
            SpreadDirection::Right,
            SpreadDirection::Up,
            SpreadDirection::Down,
        ]
    }

    fn to_vec2(&self) -> Vec2 {
        match self {
            SpreadDirection::Left => Vec2::NEG_X,
            SpreadDirection::Right => Vec2::X,
            SpreadDirection::Up => Vec2::Y,
            SpreadDirection::Down => Vec2::NEG_Y,
        }
    }

    fn to_indices(&self, x_index: usize, y_index: usize) -> Option<(usize, usize)> {
        match self {
            SpreadDirection::Left => Some((rq!(x_index.checked_sub(1)), y_index)),
            SpreadDirection::Right => Some((rq!(checked_add_one(x_index)), y_index)),
            SpreadDirection::Up => Some((x_index, rq!(y_index.checked_sub(1)))),
            SpreadDirection::Down => Some((x_index, rq!(checked_add_one(y_index)))),
        }
    }
}

fn checked_add_one(value: usize) -> Option<usize> {
    if value >= MAX_FIRE_INDEX - 1 {
        None
    } else {
        Some(value + 1)
    }
}

#[derive(Component, Debug, Clone, Deref, DerefMut, PartialEq, Eq)]
struct EndTimer(pub Timer);

#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct FireGrid {
    fire_grid: [[bool; MAX_FIRE_INDEX]; MAX_FIRE_INDEX],
}

impl FireGrid {
    fn new() -> Self {
        Self {
            fire_grid: [[false; MAX_FIRE_INDEX]; MAX_FIRE_INDEX],
        }
    }

    fn get_value(&self, x_index: usize, y_index: usize) -> bool {
        self.fire_grid[y_index][x_index]
    }

    fn set_value(&mut self, x_index: usize, y_index: usize) {
        self.fire_grid[y_index][x_index] = true;
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct FireSpread {
    directions_left: Vec<SpreadDirection>,
    timer: Timer,
    x_index: usize,
    y_index: usize,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EndSequenceAssets {
    #[asset(path = "image/fire_spritesheet.png")]
    pub fire_spritesheet_image: Handle<Image>,
}

impl Configure for EndSequenceAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EndSequenceSystems;

impl Configure for EndSequenceSystems {
    fn configure(app: &mut App) {
        app.configure_sets(Update, EndSequenceSystems.run_if(EndSequence::is_enabled));
    }
}

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EndSequencePausedSystems;

impl Configure for EndSequencePausedSystems {
    fn configure(app: &mut App) {
        app.configure_sets(
            Update,
            EndSequencePausedSystems.run_if(EndSequence::is_disabled),
        );
    }
}

fn handle_end_timer(
    mut end_timer: Single<(Entity, &mut EndTimer)>,
    time: Res<Time>,
    mut gameover_menu: NextMut<ShowGameOverMenu>,
    mut commands: Commands,
) {
    end_timer.1.tick(time.delta());

    if end_timer.1.just_finished() {
        gameover_menu.enable_default();
        commands.entity(end_timer.0).despawn();
    }
}

fn start_end_sequence(
    mut event_reader: EventReader<StartEndSequenceEvent>,
    mut end_sequence_state: NextMut<EndSequence>,
    mut commands: Commands,
    assets: Res<EndSequenceAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    goal_pos: Single<&GlobalTransform, With<Goal>>,
) {
    if event_reader.read().last().is_some() {
        end_sequence_state.enable_default();

        let mut fire_grid = FireGrid::new();
        fire_grid.set_value(MAX_FIRE_INDEX / 2, MAX_FIRE_INDEX / 2);

        create_fire(
            Vec2::new(goal_pos.translation().x, goal_pos.translation().y),
            MAX_FIRE_INDEX / 2,
            MAX_FIRE_INDEX / 2,
            &mut commands,
            &assets,
            &mut texture_atlas_layouts,
        );

        commands.spawn(fire_grid);

        commands.spawn(EndTimer(Timer::from_seconds(10.0, TimerMode::Once)));
    }
}

fn process_fire_spread(
    fire_spread_query: Query<(&mut FireSpread, &Transform)>,
    mut fire_grid: Single<&mut FireGrid>,
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<EndSequenceAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut thread_rng = rand::thread_rng();
    for (mut fire_spread, fire_transform) in fire_spread_query {
        fire_spread.timer.tick(time.delta());

        if fire_spread.timer.just_finished() {
            if !fire_spread.directions_left.is_empty() {
                let index = thread_rng.gen_range(0..fire_spread.directions_left.len());
                let direction = fire_spread.directions_left.remove(index);

                let position =
                    fire_transform.translation.xy() + direction.to_vec2() * FIRE_SPREAD_OFFSET;

                let new_indices = direction.to_indices(fire_spread.x_index, fire_spread.y_index);

                if let Some(new_indices) = new_indices {
                    if !fire_grid.get_value(new_indices.0, new_indices.1) {
                        fire_grid.set_value(new_indices.0, new_indices.1);
                        create_fire(
                            position,
                            new_indices.0,
                            new_indices.1,
                            &mut commands,
                            &assets,
                            &mut texture_atlas_layouts,
                        );
                    }
                }
            }
        }
    }
}

fn create_fire(
    position: Vec2,
    x_index: usize,
    y_index: usize,
    commands: &mut Commands,
    assets: &Res<EndSequenceAssets>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout =
        TextureAtlasLayout::from_grid(UVec2::splat(50), FIRE_INDICES as u32, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let fire_sprite = Sprite::from_atlas_image(
        assets.fire_spritesheet_image.clone(),
        TextureAtlas {
            layout: texture_atlas_layout,
            index: 0,
        },
    );

    commands.spawn((
        FireSpread {
            directions_left: SpreadDirection::all(),
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            x_index,
            y_index,
        },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        AnimationIndices::new(0, FIRE_INDICES - 1),
        fire_sprite,
        Transform::from_translation(position.extend(100.0)),
    ));
}

fn reset_end_sequence(
    mut commands: Commands,
    fire_grid_query: Query<Entity, With<FireGrid>>,
    fire_spread_query: Query<Entity, With<FireSpread>>,
    mut end_sequence_state: NextMut<EndSequence>,
    mut gameover_menu: NextMut<ShowGameOverMenu>,
) {
    end_sequence_state.disable();
    gameover_menu.disable();

    for entity in fire_grid_query.iter().chain(fire_spread_query.iter()) {
        commands.entity(entity).despawn();
    }
}
