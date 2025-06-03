use crate::prelude::*;

mod player;
mod animated_sprite;
mod movement;
mod level;
mod chain;
mod chain_movement;

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameAssets {
    #[asset(path = "audio/music/545458__bertsz__bit-forest-evil-theme-music.ogg")]
    music: Handle<AudioSource>,
}

impl Configure for GameAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player::plugin,
        level::plugin,
        movement::plugin,
        animated_sprite::plugin,
        chain::plugin,
        chain_movement::plugin,
    ));

    app.configure::<GameAssets>();
}