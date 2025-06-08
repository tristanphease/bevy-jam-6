use bevy_ecs_ldtk::prelude::*;

use crate::game::death_anim::PauseWhenDyingSystems;
use crate::game::end_sequence::StartEndSequenceEvent;
use crate::game::player::Player;
use crate::prelude::*;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<GoalBundle>("goal");

    app.add_event::<EnableGoalEvent>();

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((process_goals, rotate, update_goal))
            .in_set(PausableSystems)
            .in_set(PauseWhenDyingSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Goal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct DisabledGoal;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct RotateComponent;

#[derive(Bundle, Default, LdtkEntity)]
pub struct GoalBundle {
    goal: Goal,
    sensor: Sensor,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

#[derive(Debug, Default, Event)]
pub struct EnableGoalEvent;

fn update_goal(
    mut commands: Commands,
    mut event_reader: EventReader<EnableGoalEvent>,
    goal_query: Query<(Entity, &mut Sprite), With<Goal>>,
) {
    if event_reader.read().last().is_some() {
        for (goal_entity, mut goal_sprite) in goal_query {
            goal_sprite.color = Color::default();

            commands
                .entity(goal_entity)
                .remove::<DisabledGoal>()
                .insert(RotateComponent)
                .observe(goal_observer);
        }
    }
}

fn process_goals(mut commands: Commands, goal_query: Query<(Entity, &mut Sprite), Added<Goal>>) {
    for (goal_entity, mut goal_sprite) in goal_query {
        goal_sprite.color = Color::Hsla(Hsla::default().with_alpha(0.2));

        commands
            .entity(goal_entity)
            .insert((Collider::rectangle(10.0, 10.0), CollisionEventsEnabled))
            .insert(DisabledGoal);
    }
}

fn goal_observer(
    trigger: Trigger<OnCollisionStart>,
    player_query: Query<(Entity, &mut Transform), With<Player>>,
    level_selection: ResMut<LevelSelection>,
    mut event_writer: EventWriter<StartEndSequenceEvent>,
) {
    let entity = trigger.collider;

    if player_query.contains(entity) {
        let indices = match level_selection.into_inner() {
            LevelSelection::Indices(indices) => indices,
            _ => panic!("level selection should always be Indices in this game"),
        };

        if indices.level < 2 {
            indices.level += 1;
        } else {
            event_writer.write(StartEndSequenceEvent);
        }
    }
}

fn rotate(query: Query<&mut Transform, With<RotateComponent>>, time: Res<Time>) {
    for mut object_transform in query {
        object_transform.rotate_z(time.delta_secs() * 1.0);
    }
}
