use crate::prelude::*;

mod animated_sprite;
mod boxes;
mod chain;
mod chain_movement;
pub mod death_anim;
mod goal;
mod level;
mod movement;
mod player;
mod player_chain;
mod vines;

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
        goal::plugin,
        vines::plugin,
        player_chain::plugin,
        death_anim::plugin,
        boxes::plugin,
    ));

    app.configure::<GameAssets>();
}
