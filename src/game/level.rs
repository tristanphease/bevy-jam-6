use crate::{prelude::*, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(StateFlush, Screen::Gameplay.on_enter(spawn_level));
} 



#[derive(Bundle)]
pub struct StaticWallBundle {
    body: RigidBody,
    collider: Collider,
}

fn spawn_level(
    mut commands: Commands,
) {
    let width = 1000.0;
    let height = 200.0;
    commands.spawn((
        Sprite {
            color: Color::hsl(260.0, 0.95, 0.7),
            custom_size: Some(Vec2::new(width, height)),
            ..default()
        },
        RigidBody::Static,
        Transform::from_xyz(0.0, -200.0, 0.0),
        Collider::rectangle(width, height)
    ));
}