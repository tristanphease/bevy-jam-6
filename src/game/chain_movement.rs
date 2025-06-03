use crate::{game::{chain::{ChainImmunity, ChildOfChain, ChildrenOfChain, ConnectedChain}, player::Player}, prelude::*, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerChainEvent>();

    app.add_systems(
        Update, 
        Screen::Gameplay.on_update((
            handle_keyboard_input,
            handle_player_chain_event,
        ))
    );
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerChainEvent {
    LeaveChain,
}

fn handle_keyboard_input(
    mut player_chain_event: EventWriter<PlayerChainEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    connected_chain: Single<Has<ConnectedChain>, With<Player>>,
) {
    if !*connected_chain {
        return;
    }
    let leave_chain = keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space]);

    if leave_chain {
        player_chain_event.write(PlayerChainEvent::LeaveChain);
    }
}

fn handle_player_chain_event(
    mut chain_event_reader: EventReader<PlayerChainEvent>,
    mut commands: Commands,
    mut player_query: Single<(Entity, &mut Transform, &mut LinearVelocity, &ChildOfChain), (With<Player>, With<ConnectedChain>)>,
    chain_parent_query: Query<&LinearVelocity, (With<ChildrenOfChain>, Without<ConnectedChain>)>,
) {
    for chain_event in chain_event_reader.read() {
        match chain_event {
            PlayerChainEvent::LeaveChain => {

                let chain_linear_velocity = chain_parent_query.get(player_query.3.parent()).unwrap();

                commands.entity(player_query.0)
                    .remove::<ConnectedChain>()
                    .remove::<ChildOfChain>()
                    .insert(GravityScale(2.0))
                    .insert(ChainImmunity(Timer::from_seconds(1.0, TimerMode::Once)));

                player_query.1.rotation = Quat::default();
                *player_query.2 = *chain_linear_velocity;

                
            },
        }
    }
}



