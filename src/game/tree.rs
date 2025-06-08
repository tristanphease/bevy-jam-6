use bevy_ecs_ldtk::prelude::*;

use crate::game::chain_movement::GameLayer;
use crate::game::death_anim::PlayerDeath;
use crate::game::player::Player;
use crate::game::vines::KillsPlayer;
use crate::game::vines::on_collision_kills_player;
use crate::prelude::*;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<TreeBundle>("tree");

    app.configure::<TreeAssets>();

    app.add_systems(
        Update,
        Screen::Gameplay.on_update((process_tree, process_apple_timer)),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Tree;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct AppleTimer(pub Timer);

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TreeAssets {
    #[asset(path = "image/apple.png")]
    apple_image: Handle<Image>,
}

impl Configure for TreeAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

#[derive(Bundle, Default, LdtkEntity)]
struct TreeBundle {
    tree: Tree,
    #[sprite_sheet]
    sprite: Sprite,
    kills_player: KillsPlayer,

    rigid_body: RigidBody,
    sensor: Sensor,
    collision_events_enabled: CollisionEventsEnabled,
}

fn process_tree(tree_added_query: Query<Entity, Added<Tree>>, mut commands: Commands) {
    for tree_entity in tree_added_query {
        commands
            .entity(tree_entity)
            .insert(Collider::rectangle(80.0, 80.0))
            .insert(AppleTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
            .insert(CollisionLayers::new(
                GameLayer::TreeLayer,
                LayerMask::DEFAULT,
            ))
            .observe(on_collision_kills_player);
    }
}

fn process_apple_timer(
    apple_timer_query: Query<(&mut AppleTimer, &GlobalTransform)>,
    player_transform: Single<&GlobalTransform, With<Player>>,
    mut commands: Commands,
    time: Res<Time>,
    tree_assets: Res<TreeAssets>,
) {
    for (mut apple_timer, transform) in apple_timer_query {
        apple_timer.tick(time.delta());
        if apple_timer.just_finished() {
            let x = 6.0 * (random::<f32>() - 0.5);
            let y = 6.0 * (random::<f32>() - 0.5) + 5.0;
            let apple_pos = transform.translation() + Vec3::new(x, y, 0.0);

            let direction = player_transform.translation().xy() - apple_pos.xy();
            let x_velocity = f32::min(direction.x, 3000.0);
            let y_velocity = 140.0;
            let linear_velocity = LinearVelocity(Vec2::new(x_velocity, y_velocity));

            commands
                .spawn((
                    Apple,
                    Sprite {
                        image: tree_assets.apple_image.clone(),
                        custom_size: Some(Vec2::splat(10.0)),
                        ..default()
                    },
                    Transform::from_translation(apple_pos),
                    linear_velocity,
                    AngularVelocity(0.1),
                    Collider::circle(2.0),
                    RigidBody::Dynamic,
                    CollisionEventsEnabled,
                    CollisionLayers::new(
                        LayerMask::DEFAULT,
                        LayerMask::DEFAULT & !(GameLayer::TreeLayer.to_bits()),
                    ),
                ))
                .observe(on_apple_collision);
        }
    }
}

fn on_apple_collision(
    trigger: Trigger<OnCollisionStart>,
    player_query: Query<Entity, With<Player>>,
    tree_query: Query<Entity, With<Tree>>,
    mut death_event_writer: EventWriter<PlayerDeath>,
    mut commands: Commands,
) {
    let apple_entity = trigger.target();
    let other_entity = trigger.collider;

    if player_query.contains(other_entity) {
        death_event_writer.write(PlayerDeath);
    } else if !tree_query.contains(other_entity) {
        // delete apple if we collide with anything else
        commands.entity(apple_entity).despawn();
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Apple;
