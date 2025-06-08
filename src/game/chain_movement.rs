use crate::{
    game::{
        chain::{ChainImmunity, ChainJoint, ConnectedChain},
        player::Player,
    },
    prelude::*,
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PlayerChainEvent>();

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((handle_keyboard_input, handle_player_chain_event))
            .in_set(PausableSystems),
    );
}

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    ChainLayer, // for chains
    TreeLayer,  // for trees
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
    let leave_chain =
        keyboard_input.any_just_pressed([KeyCode::KeyW, KeyCode::ArrowUp, KeyCode::Space]);

    if leave_chain {
        player_chain_event.write(PlayerChainEvent::LeaveChain);
    }
}

fn handle_player_chain_event(
    mut chain_event_reader: EventReader<PlayerChainEvent>,
    mut commands: Commands,
    mut player_query: Single<
        (Entity, &mut Transform, &mut LinearVelocity, &ConnectedChain),
        With<Player>,
    >,
    joint_query: Query<(Entity, &DistanceJoint), With<ChainJoint>>,
) {
    for chain_event in chain_event_reader.read() {
        match chain_event {
            PlayerChainEvent::LeaveChain => {
                // let chain_linear_velocity = chain_parent_query.get(player_query.3.parent()).unwrap();
                let player_entity = player_query.0;
                commands
                    .entity(player_entity)
                    .remove::<ConnectedChain>()
                    .insert(ChainImmunity::new(
                        Timer::from_seconds(1.0, TimerMode::Once),
                        player_query.3.0.to_string(),
                    ))
                    .insert(GravityScale(2.0))
                    .insert(CollisionLayers::DEFAULT);

                player_query.1.rotation = Quat::default();

                for joints in joint_query {
                    let joint = joints.1;
                    if joint.entity1 == player_entity || joint.entity2 == player_entity {
                        commands.entity(joints.0).despawn();
                    }
                }
            },
        }
    }
}
