use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::Indices;
use bevy::render::mesh::PrimitiveTopology;

use crate::game::player::Player;
use crate::game::player::PlayerEye;
use crate::menu::Menu;
use crate::prelude::*;
use crate::screen::Screen;
use crate::screen::gameplay::ShowPlayerDeathMenu;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerDeath>();

    app.add_state::<PlayerDying>();

    app.configure_sets(
        Update,
        PauseWhenDyingSystems.run_if(PlayerDying::is_disabled),
    );

    app.add_systems(StateFlush, Menu::Death.on_enter(reset_death));

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((handle_player_death, handle_player_death_timer))
            .in_set(PausableSystems),
    );
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerDeath;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct DyingTimer(pub Timer);

#[derive(State, Reflect, Copy, Clone, Default, Eq, PartialEq, Debug)]
#[reflect(Resource)]
pub struct PlayerDying;

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PauseWhenDyingSystems;

fn handle_player_death(
    mut player_death_event_reader: EventReader<PlayerDeath>,
    mut player_dying: NextMut<PlayerDying>,
    player_entity: Single<Entity, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_eye_query: Query<Entity, With<PlayerEye>>,
) {
    if let Some(_death_event) = player_death_event_reader.read().last() {
        if player_dying.get().is_none() {
            player_dying.enable_default();

            commands
                .entity(*player_entity)
                .insert(DyingTimer(Timer::from_seconds(3.0, TimerMode::Once)));

            let new_eye_mesh = meshes.add(generate_cross_mesh());
            for eye_entity in player_eye_query {
                commands
                    .entity(eye_entity)
                    .insert(Mesh2d(new_eye_mesh.clone()));
            }
        }
    }
}

// <https://bevy.org/examples/2d-rendering/mesh2d-manual/>
// adapted to make a cross which is two diagonal rectangles
fn generate_cross_mesh() -> Mesh {
    let mut cross_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    let vertices_positions = vec![
        [-10.0, 6.0, 0.0],
        [-6.0, 10.0, 0.0],
        [10.0, -6.0, 0.0],
        [6.0, -10.0, 0.0],
        [6.0, 10.0, 0.0],
        [10.0, 6.0, 0.0],
        [-6.0, -10.0, 0.0],
        [-10.0, -6.0, 0.0],
    ];

    cross_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices_positions);

    let triangle_indices = vec![0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];
    cross_mesh.insert_indices(Indices::U32(triangle_indices));

    cross_mesh
}

fn handle_player_death_timer(
    time: Res<Time>,
    dying_timer_query: Query<&mut DyingTimer>,
    mut player_death_menu: NextMut<ShowPlayerDeathMenu>,
) {
    for mut timer in dying_timer_query {
        timer.tick(time.delta());

        if timer.just_finished() {
            player_death_menu.enable_default();
        }
    }
}

fn reset_death(
    mut player_dying: NextMut<PlayerDying>,
    mut player_death_menu: NextMut<ShowPlayerDeathMenu>,
) {
    player_dying.disable();
    player_death_menu.disable();
}
