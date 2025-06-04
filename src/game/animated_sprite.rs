use crate::{prelude::*, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, Screen::Gameplay.on_update(animate_sprite));
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationIndices {
    first: usize,
    last: usize,
}

impl AnimationIndices {
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }

    pub fn single(index: usize) -> Self {
        Self {
            first: index,
            last: index,
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut AnimationTimer, &AnimationIndices)>,
) {
    for (mut sprite, mut timer, indices) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index >= indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
