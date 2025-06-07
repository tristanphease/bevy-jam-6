use bevy_ecs_ldtk::prelude::*;

use crate::{
    game::{death_anim::PauseWhenDyingSystems, player::Player},
    prelude::*,
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<GoalBundle>("goal");

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((process_goals, rotate))
            .in_set(PausableSystems)
            .in_set(PauseWhenDyingSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Goal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct RotateComponent;

#[derive(Bundle, Default, LdtkEntity)]
pub struct GoalBundle {
    goal: Goal,
    sensor: Sensor,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    rotate: RotateComponent,
}

fn process_goals(mut commands: Commands, goal_query: Query<Entity, Added<Goal>>) {
    for goal_entity in goal_query {
        commands
            .entity(goal_entity)
            .insert((Collider::rectangle(10.0, 10.0), CollisionEventsEnabled))
            .observe(goal_observer);
    }
}

fn goal_observer(
    trigger: Trigger<OnCollisionStart>,
    player_query: Query<(Entity, &mut Transform), With<Player>>,
    level_selection: ResMut<LevelSelection>,
) {
    let entity = trigger.collider;

    if player_query.contains(entity) {
        let indices = match level_selection.into_inner() {
            LevelSelection::Indices(indices) => indices,
            _ => panic!("level selection should always be Indices in this game"),
        };

        indices.level += 1;
    }
}

fn rotate(query: Query<&mut Transform, With<RotateComponent>>, time: Res<Time>) {
    for mut object_transform in query {
        object_transform.rotate_z(time.delta_secs() * 1.0);
    }
}
