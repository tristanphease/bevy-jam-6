use bevy_ecs_ldtk::prelude::*;

use crate::{
    game::{death_anim::PlayerDeath, player::Player},
    prelude::*,
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_int_cell_for_layer::<VinesBundle>("vines", 1);

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update(process_vines)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct KillsPlayer;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Vine;

#[derive(Bundle, Default, LdtkIntCell)]
struct VinesBundle {
    vine: Vine,
    kills_player: KillsPlayer,

    rigid_body: RigidBody,
    sensor: Sensor,
    collision_events_enabled: CollisionEventsEnabled,
}

fn process_vines(vines_added_query: Query<Entity, Added<Vine>>, mut commands: Commands) {
    for vine_entity in vines_added_query {
        commands
            .entity(vine_entity)
            .insert(Collider::rectangle(5.0, 5.0))
            .observe(on_vine_collide);
    }
}

fn on_vine_collide(
    trigger: Trigger<OnCollisionStart>,
    player_query: Query<Entity, With<Player>>,
    mut death_event_writer: EventWriter<PlayerDeath>,
) {
    let other_entity = trigger.collider;
    if player_query.contains(other_entity) {
        death_event_writer.write(PlayerDeath);
    }
}
