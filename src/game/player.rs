use crate::{prelude::*, screen::Screen};

use super::movement::CharacterControllerBundle;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Player;

const PLAYER_WIDTH: f32 = 200.0;
const PLAYER_HEIGHT: f32 = 200.0;

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[asset(path = "image/player.png")]
    player: Handle<Image>,
}

impl Configure for PlayerAssets {
    fn configure(app: &mut App) {
        app.register_type::<Self>();
        app.init_collection::<Self>();
    }
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.configure::<PlayerAssets>();

    app.add_systems(StateFlush, Screen::Gameplay.on_enter(spawn_player));
} 

fn spawn_player(mut commands: Commands,
    assets: Res<PlayerAssets>) {

    let mut player_sprite = Sprite::from_image(assets.player.clone());
    player_sprite.custom_size = Some((PLAYER_WIDTH, PLAYER_HEIGHT).into());
    
    commands.spawn((
        player_sprite,
        Player,
        Transform::from_xyz(0.0, 100.0, 0.0),
        CharacterControllerBundle::new(Collider::rectangle(PLAYER_WIDTH, PLAYER_HEIGHT)),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        ColliderDensity(2.0),
        GravityScale(1.5),
    ));
}
