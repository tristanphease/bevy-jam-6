use crate::{game::chain_movement::GameLayer, prelude::*, screen::Screen};
use avian2d::math::Vector;
use bevy_ecs_ldtk::prelude::*;

const CHAIN_SIZE: f32 = 0.16;
const CHAIN_IMAGE_SIZE: f32 = 100.0;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<ChainImportBundle>("chain");

    app.configure::<ChainAssets>();

    app.add_systems(Update, Screen::Gameplay.on_update((
        process_chain,
        process_chain_immunity_timer,
    )));
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChainAssets {
    #[asset(path = "image/chain.png")]
    chain_image: Handle<Image>,
}

impl Configure for ChainAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

// components for other entities to add to interact with chain

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct CanAttachChain;

/// Means the chain handles the movement now
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ConnectedChain;

/// Means the entity is immune from attaching to chains for a time defined by the timer
#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct ChainImmunity(pub Timer);

/// Part of a chain
#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ChainPart;

/// Chain that's imported from the map editor
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct ChainImport;

/// Chain joint attached between player and chain
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ChainJoint;

#[derive(Bundle, Default, LdtkEntity)]
pub struct ChainImportBundle {
    chain_import: ChainImport,
}

#[derive(Bundle, Default)]
struct ChainBundle {
    sprite: Sprite,
    rigid_body: RigidBody,
    collider: Collider,
    transform: Transform,
    mass_properties_bundle: MassPropertiesBundle,
    collision_event_enabled: CollisionEventsEnabled,
    chain_part: ChainPart,
    chain_layer: CollisionLayers,
}

impl ChainBundle {
    pub fn new(image_handle: Handle<Image>, 
        rigid_body: RigidBody, 
        transform: Transform,
        chain_part: ChainPart,
    ) -> Self {
        Self {
            sprite: Sprite::from_image(image_handle),
            rigid_body,
            collider: Collider::rectangle(CHAIN_SIZE * 10.0, CHAIN_SIZE * 50.0),
            transform,
            mass_properties_bundle: MassPropertiesBundle::from_shape(&Rectangle::new(10.0, 50.0), 0.02),
            collision_event_enabled: CollisionEventsEnabled,
            chain_part,
            chain_layer: CollisionLayers::new(GameLayer::ChainLayer, LayerMask::ALL),
        }
    }
}

// process and create the chain when imported
fn process_chain(
    mut commands: Commands,
    chain_query: Query<&Transform, Added<ChainImport>>,
    chain_assets: Res<ChainAssets>,
) {
    
    for chain_transform in chain_query.iter() {
        let mut last_chain_option: Option<Entity> = None;
        let max_value = f32::floor(chain_transform.scale.y / CHAIN_SIZE);
        let max_value_i32 = max_value as i32;
        for y in 0..max_value_i32 {
            // let mut transform = Transform::from_translation(chain_transform.translation);
            let mut transform = chain_transform.clone();
            transform.scale.y = CHAIN_SIZE;
            transform.translation.y -= (y as f32) * CHAIN_SIZE * CHAIN_IMAGE_SIZE;
            if let Some(last_chain) = last_chain_option {
                let next_chain = commands.spawn(
                    ChainBundle::new(
                        chain_assets.chain_image.clone(),
                            RigidBody::Dynamic,
                            transform,
                            ChainPart,
                    ))
                    .observe(observe_chain_collision)
                    .id();

                // joint between the two entities
                commands.spawn(
                    RevoluteJoint::new(last_chain, next_chain)
                        .with_local_anchor_2(Vector::Y * 1.0 * CHAIN_SIZE * CHAIN_IMAGE_SIZE)
                        .with_angle_limits(-0.01, 0.01)
                        .with_compliance(0.000001)
                );

                last_chain_option = Some(next_chain);
            } else {
                // spawn fixed chain at the start
                // could use different sprite for this one to indicate it's fixed
                last_chain_option = Some(commands.spawn(ChainBundle::new(
                        chain_assets.chain_image.clone(),
                        RigidBody::Kinematic,
                        transform,
                        ChainPart,
                ))
                .observe(observe_chain_collision)
                .id());
            }
        }
        
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
#[relationship(relationship_target = ChildrenOfChain)]
pub struct ChildOfChain(#[entities] pub Entity);

#[derive(Component, Default, Debug, Deref, DerefMut, PartialEq, Eq)]
#[relationship_target(relationship = ChildOfChain)]
pub struct ChildrenOfChain(Vec<Entity>);

fn observe_chain_collision(
    trigger: Trigger<OnCollisionStart>, 
    mut commands: Commands,
    attachable_query: Query<Entity, (With<CanAttachChain>, Without<ConnectedChain>, Without<ChainImmunity>)>,
) {
    let chain = trigger.target();
    let other_entity = trigger.collider;
    if attachable_query.contains(other_entity) {

        // create filter so that we don't collide with the chain while on it
        let filters = *LayerMask::ALL & !(GameLayer::ChainLayer.to_bits());
        let ignore_chain_collision_layer = CollisionLayers::new(
            LayerMask::DEFAULT, 
            filters,
        );

        commands.entity(other_entity)
            .insert(ConnectedChain)
            .insert(ChildOfChain(chain))
            .insert(GravityScale(1.0))
            .insert(ignore_chain_collision_layer); 

        commands.spawn((
            DistanceJoint::new(chain, other_entity)
                .with_limits(1.0, 5.0),
            ChainJoint,
        ));
    }
}

fn process_chain_immunity_timer(
    time: Res<Time>,
    mut commands: Commands,
    chain_immunity_query: Query<(&mut ChainImmunity, Entity)>
) {
    for mut chain_immunity in chain_immunity_query {
        chain_immunity.0.tick(time.delta());
        if chain_immunity.0.finished() {
            commands.entity(chain_immunity.1)
                .remove::<ChainImmunity>();
        }
    }
}