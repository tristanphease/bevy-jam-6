use crate::{
    game::player::Player,
    menu::Menu,
    prelude::*,
    screen::{Screen, gameplay::ShowPlayerDeathMenu},
};

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
) {
    if let Some(_death_event) = player_death_event_reader.read().last() {
        if player_dying.get().is_none() {
            info!("enabling death");
            player_dying.enable_default();

            commands
                .entity(*player_entity)
                .insert(DyingTimer(Timer::from_seconds(3.0, TimerMode::Once)));
        }
    }
}

fn handle_player_death_timer(
    time: Res<Time>,
    dying_timer_query: Query<&mut DyingTimer>,
    mut player_death_menu: NextMut<ShowPlayerDeathMenu>,
) {
    for mut timer in dying_timer_query {
        timer.tick(time.delta());

        if timer.just_finished() {
            info!("enabling death menu");
            player_death_menu.enable_default();
        }
    }
}

fn reset_death(
    mut player_dying: NextMut<PlayerDying>,
    mut player_death_menu: NextMut<ShowPlayerDeathMenu>,
) {
    info!("resetting death");
    player_dying.disable();
    player_death_menu.disable();
}
