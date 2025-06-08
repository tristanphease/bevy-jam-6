use bevy_ecs_ldtk::prelude::*;

use crate::prelude::*;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<TempTextBundle>("text");

    app.add_systems(Update, Screen::Gameplay.on_update(process_text));
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct WorldText(pub String);

impl WorldText {
    fn from_entity_instance(entity_instance: &EntityInstance) -> Self {
        Self(entity_instance.get_string_field("text").unwrap().clone())
    }
}

#[derive(Bundle, Default, LdtkEntity)]
struct TempTextBundle {
    #[with(WorldText::from_entity_instance)]
    world_text: WorldText,
}

fn process_text(text_query: Query<(Entity, Ref<WorldText>)>, mut commands: Commands) {
    for (text_entity, world_text) in text_query {
        if world_text.is_added() {
            commands
                .entity(text_entity)
                .insert(Text2d((*world_text).to_string()));
        }
    }
}
