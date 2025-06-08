use bevy_ecs_ldtk::prelude::*;

use crate::{
    game::{goal::EnableGoalEvent, player::Player},
    prelude::*,
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<BoxesBundle>("box");

    app.add_systems(Update, Screen::Gameplay.on_update(spawn_score));
    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((process_boxes, update_score_text))
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Boxes;

#[derive(Bundle, Default, LdtkEntity)]
pub struct BoxesBundle {
    boxes: Boxes,
    #[sprite_sheet]
    sprite: Sprite,

    rigid_body: RigidBody,
    sensor: Sensor,
    collision_events_enabled: CollisionEventsEnabled,
}

#[derive(Component, Default)]
struct BoxInfo {
    collected: i32,
    total: i32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ScoreText;

fn process_boxes(
    mut commands: Commands,
    box_query: Query<Entity, Added<Boxes>>,
    mut box_info: Single<&mut BoxInfo>,
) {
    for box_entity in box_query {
        commands
            .entity(box_entity)
            .insert(Collider::rectangle(15.0, 15.0))
            .observe(on_box_collect);

        box_info.total += 1;
    }
}

fn on_box_collect(
    trigger: Trigger<OnCollisionStart>,
    player_query: Query<Entity, With<Player>>,
    mut box_info: Single<&mut BoxInfo>,
    mut commands: Commands,
    mut goal_event_writer: EventWriter<EnableGoalEvent>,
) {
    let box_entity = trigger.body;
    let other_entity = trigger.collider;

    if player_query.contains(other_entity) {
        if let Some(box_entity) = box_entity {
            box_info.collected += 1;
            commands.entity(box_entity).despawn();

            if box_info.collected >= box_info.total {
                goal_event_writer.write(EnableGoalEvent);
            }
        }
    }
}

fn spawn_score(mut commands: Commands, level_entity: Single<Entity, Added<LevelIid>>) {
    let box_info = BoxInfo::default();

    commands.spawn((
        Text::new(format!("{}/{}", box_info.collected, box_info.total)),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(2.0),
            left: Val::Px(2.0),
            ..default()
        },
        ScoreText,
    ));

    commands.entity(*level_entity).with_children(|level| {
        level.spawn(box_info);
    });
}

fn update_score_text(
    score_text_query: Query<&mut Text, With<ScoreText>>,
    box_info: Single<&BoxInfo>,
) {
    for mut score_text in score_text_query {
        score_text.0 = format!("{}/{}", box_info.collected, box_info.total);
    }
}
