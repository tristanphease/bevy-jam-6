//! <https://github.com/Jondolf/avian/blob/main/crates/avian2d/examples/dynamic_character_2d/plugin.rs>

use avian2d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;
use pyri_state::pattern::StatePattern;

use crate::{
    game::{
        chain::ConnectedChain,
        death_anim::PauseWhenDyingSystems,
        player::{Player, PlayerState},
    },
    screen::Screen,
};

use crate::prelude::*;

use super::player::{ChangePlayerDirection, ChangePlayerState};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<MovementAction>();
    app.add_systems(
        Update,
        Screen::Gameplay
            .on_update((
                handle_keyboard_input,
                update_grounded,
                control_movement,
                apply_movement_damping,
                update_idle,
            ))
            .in_set(PausableSystems)
            .in_set(PauseWhenDyingSystems),
    );

    // for debugging
    // app.insert_gizmo_config(PhysicsGizmos::default(), GizmoConfig::default());
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The maximum velocity a player can reach from just movement keys
#[derive(Component, Deref)]
pub struct MaxPlayerVelocity(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// The maximum angle a slope can have for a character controller
/// to be able to climb and jump. If the slope is steeper than this angle,
/// the character will slide down.
#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    max_slope_angle: MaxSlopeAngle,
    max_player_velocity: MaxPlayerVelocity,
}

impl MovementBundle {
    pub fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        max_slope_angle: Scalar,
        max_player_velocity: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
            max_player_velocity: MaxPlayerVelocity(max_player_velocity),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(2000.0, 0.95, 300.0, avian2d::math::PI * 0.45, 300.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        // this really should be 0.99 but then it's way too large, idk why
        caster_shape.set_scale(Vector::ONE * 0.12, 10);

        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(5.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }
}

#[derive(Event)]
pub enum MovementAction {
    Move(Scalar),
    Jump,
}

fn handle_keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = if right && !left {
        1
    } else if left && !right {
        -1
    } else {
        0
    };

    if horizontal != 0 {
        movement_event_writer.write(MovementAction::Move(horizontal as Scalar));
    }

    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW]) {
        movement_event_writer.write(MovementAction::Jump);
    }
}

fn control_movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controller_query: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        &MaxPlayerVelocity,
        Has<Grounded>,
        Has<ConnectedChain>,
    )>,
    mut player_direction_writer: EventWriter<ChangePlayerDirection>,
    mut player_state_writer: EventWriter<ChangePlayerState>,
) {
    let delta_time = time.delta_secs();

    for event in movement_event_reader.read() {
        for (
            movement_acceleration,
            jump_impulse,
            mut linear_velocity,
            max_velocity,
            is_grounded,
            is_on_chain,
        ) in &mut controller_query
        {
            let jump_damper = if is_grounded { 1.0 } else { 0.6 };
            let damper = if is_on_chain { 0.6 } else { 1.0 } * jump_damper;

            match event {
                MovementAction::Move(direction) => {
                    if *direction > 0.0 && linear_velocity.x < **max_velocity
                        || *direction < 0.0 && linear_velocity.x > -**max_velocity
                    {
                        linear_velocity.x +=
                            damper * *direction * movement_acceleration.0 * delta_time;
                    }

                    player_state_writer.write(ChangePlayerState::ChangeModeRunning);
                    let new_direction = if *direction < 0.0 {
                        ChangePlayerDirection::TurnLeft
                    } else {
                        ChangePlayerDirection::TurnRight
                    };
                    player_direction_writer.write(new_direction);
                },
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0 * damper;
                        player_state_writer.write(ChangePlayerState::ChangeModeJumping);
                    }
                },
            }
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &ShapeHits,
            &Rotation,
            &PlayerState,
            Option<&MaxSlopeAngle>,
        ),
        (With<CharacterController>, With<Player>),
    >,
    mut player_state_writer: EventWriter<ChangePlayerState>,
) {
    for (entity, hits, rotation, player_state, max_slope_angle) in &mut query {
        // The character is grounded if the shape caster has a hit with a normal
        // that isn't too steep.
        let is_grounded = hits.iter().any(|hit| {
            if let Some(angle) = max_slope_angle {
                (rotation * -hit.normal2).angle_to(Vector::Y).abs() <= angle.0
            } else {
                true
            }
        });

        if is_grounded {
            commands.entity(entity).insert(Grounded);
            if *player_state == PlayerState::Jumping {
                player_state_writer.write(ChangePlayerState::ChangeModeIdle);
            }
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn update_idle(
    player_query: Query<(&LinearVelocity, &PlayerState), With<Player>>,
    mut event_writer: EventWriter<ChangePlayerState>,
) {
    for (velocity, player_state) in player_query {
        if *player_state == PlayerState::Running && velocity.x.abs() < 0.3 {
            event_writer.write(ChangePlayerState::ChangeModeIdle);
        }
    }
}

/// Slows down movement in the X direction.
fn apply_movement_damping(
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity), Without<ConnectedChain>>,
) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
        linear_velocity.x *= damping_factor.0;
    }
}
