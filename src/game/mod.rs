use crate::{core::audio::AudioSettings, prelude::*, screen::ScreenRoot};

pub mod player;
mod movement;
mod level;

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
        movement::plugin
    ));

    app.configure::<GameAssets>();
}