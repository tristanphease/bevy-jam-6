use bevy::window::PrimaryWindow;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    game::{
        chain::{
            CHAIN_IMAGE_SIZE, CHAIN_SIZE, ChainAssets, PivotChainPart, convert_chain_to_parts,
        },
        death_anim::PauseWhenDyingSystems,
        player::Player,
    },
    prelude::*,
    screen::Screen,
};

const PLAYER_CHAIN_SIZE: f32 = 16.0;
const CHAIN_SPEED: f32 = 35.0;
const MAX_CHAIN_DIST: f32 = 100000.0;

pub(super) fn plugin(app: &mut App) {
    app.add_event::<ChainHitEnd>();
    app.add_event::<ShootChain>();

    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((
                handle_input,
                read_shoot_chain_event,
                update_shooting_chain,
                convert_chain,
                handle_despawn_timer,
            ))
            .in_set(PausableSystems)
            .in_set(PauseWhenDyingSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CanShootChain;

#[derive(Component, Debug, Clone, Copy, Default, Deref, DerefMut, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ShootingChain {
    end_position: Vec2,
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct GeneratedChain;

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct GeneratedChainJoint;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct DespawnTimer(pub Timer);

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, PartialEq, Reflect)]
#[reflect(Component)]
pub struct AttachedEntity(pub Entity);

#[derive(Component, Debug, Clone, Copy, Default, Deref, DerefMut, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ChainLength(pub f32);

#[derive(Bundle)]
pub struct ShootingChainBundle {
    shooting_chain: ShootingChain,
    attached_entity: AttachedEntity,
    current_length: ChainLength,
    sprite: Sprite,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Deref, DerefMut, Default)]
pub struct ShootChain(pub Vec2);

#[derive(Event, Debug, Clone, Copy, PartialEq, Default)]
pub struct ChainHitEnd {
    start_pos: Vec2,
    end_pos: Vec2,
}

fn convert_chain(
    mut event_reader: EventReader<ChainHitEnd>,
    existing_shooting_chain: Query<Entity, With<ShootingChain>>,
    mut commands: Commands,
    chain_assets: Res<ChainAssets>,
    level_entity: Single<Entity, With<LevelIid>>,
    generated_query: Query<Entity, With<GeneratedChain>>,
    generated_joint_query: Query<Entity, (With<RevoluteJoint>, With<GeneratedChainJoint>)>,
    generated_pivot_query: Query<Entity, (With<PivotChainPart>, With<GeneratedChain>)>,
) {
    if let Some(event) = event_reader.read().last() {
        // delete existing chain
        for entity in existing_shooting_chain {
            commands.entity(entity).despawn();

            for generated_joint_entity in generated_joint_query {
                commands.entity(generated_joint_entity).despawn();
            }

            for generated_chain_entity in generated_query {
                commands
                    .entity(generated_chain_entity)
                    .insert(DespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)));
            }

            for pivot_entity in generated_pivot_query {
                commands.entity(pivot_entity).insert(RigidBody::Dynamic);
            }

            let start_pos = event.start_pos;
            let end_pos = event.end_pos;
            convert_chain_to_parts(
                start_pos,
                end_pos,
                "player_chain",
                &mut commands,
                *level_entity,
                &chain_assets,
                true,
            );
        }
    }
}

fn update_shooting_chain(
    time: Res<Time>,
    shooting_chain_query: Query<(
        &ShootingChain,
        &mut Sprite,
        &mut Transform,
        &mut ChainLength,
        &AttachedEntity,
    )>,
    attached_entity_query: Query<(Entity, &GlobalTransform)>,
    mut chain_hit_event_writer: EventWriter<ChainHitEnd>,
    spatial_query: SpatialQuery,
) {
    for (
        shooting_chain,
        mut chain_sprite,
        mut chain_transform,
        mut chain_length,
        attached_entity,
    ) in shooting_chain_query
    {
        **chain_length += CHAIN_SPEED * time.delta_secs();

        if let Ok((attached_entity, attached_transform)) =
            attached_entity_query.get(**attached_entity)
        {
            let origin_position = attached_transform.translation().xy();
            let to_vector = (**shooting_chain - origin_position).normalize();
            let rotation_to_end_pos = Quat::from_rotation_arc(Vec3::Y, to_vector.extend(0.0));

            let chain_pos = attached_transform.translation()
                + 0.5 * (rotation_to_end_pos * Vec3::Y) * **chain_length * PLAYER_CHAIN_SIZE;

            *chain_transform = Transform {
                translation: chain_pos,
                rotation: rotation_to_end_pos,
                scale: Vec3::ONE,
            };

            chain_sprite.custom_size = Some(Vec2::new(
                PLAYER_CHAIN_SIZE,
                PLAYER_CHAIN_SIZE * **chain_length,
            ));

            let direction = Dir2::new(to_vector).unwrap();
            let filter = SpatialQueryFilter::default().with_excluded_entities([attached_entity]);
            if let Some(hit_info) = spatial_query.cast_ray(
                origin_position,
                direction,
                PLAYER_CHAIN_SIZE * **chain_length,
                false,
                &filter,
            ) {
                let hit_point = origin_position + direction * hit_info.distance;
                let end_pos =
                    hit_point - direction * hit_info.distance * CHAIN_SIZE / PLAYER_CHAIN_SIZE;
                chain_hit_event_writer.write(ChainHitEnd {
                    start_pos: hit_point,
                    end_pos,
                });
            }
        }
    }
}

fn read_shoot_chain_event(
    mut event_reader: EventReader<ShootChain>,
    mut commands: Commands,
    existing_chains: Query<Entity, With<ShootingChain>>,
    player: Single<(Entity, &GlobalTransform), With<Player>>,
    chain_assets: Res<ChainAssets>,
    spatial_query: SpatialQuery,
) {
    if let Some(event) = event_reader.read().last() {
        // kill existing chains
        for existing_chain in existing_chains {
            commands.entity(existing_chain).despawn();
        }

        let origin_point = player.1.translation().xy();
        let to_vector = (**event - origin_point).normalize();
        let direction = Dir2::new(to_vector).unwrap();

        let filter = SpatialQueryFilter::default().with_excluded_entities([player.0]);
        let end_position = if let Some(hit_info) =
            spatial_query.cast_ray(origin_point, direction, MAX_CHAIN_DIST, false, &filter)
        {
            origin_point + direction * hit_info.distance
        } else {
            origin_point + direction * MAX_CHAIN_DIST
        };

        // spawn new chain
        commands.spawn(ShootingChainBundle {
            shooting_chain: ShootingChain { end_position },
            attached_entity: AttachedEntity(player.0),
            current_length: ChainLength(0.5),
            sprite: Sprite {
                image: chain_assets.chain_image.clone(),
                image_mode: SpriteImageMode::Tiled {
                    tile_x: false,
                    tile_y: true,
                    stretch_value: PLAYER_CHAIN_SIZE / CHAIN_IMAGE_SIZE,
                },
                custom_size: Some(Vec2::splat(PLAYER_CHAIN_SIZE)),
                ..default()
            },
        });
    }
}

fn handle_input(
    player_chain: Single<Has<CanShootChain>, With<Player>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), (With<Camera2d>, With<IsDefaultUiCamera>)>,
    mut shoot_chain_event_writer: EventWriter<ShootChain>,
) {
    if !*player_chain {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(mouse_position) = window.cursor_position() {
            if let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_position) {
                shoot_chain_event_writer.write(ShootChain(world_pos));
            }
        }
    }
}

fn handle_despawn_timer(
    mut commands: Commands,
    time: Res<Time>,
    entity_despawn_timer: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut despawn_timer) in entity_despawn_timer {
        despawn_timer.tick(time.delta());
        if despawn_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
