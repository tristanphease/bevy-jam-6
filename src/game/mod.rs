use crate::prelude::*;

mod animated_sprite;
mod boxes;
mod chain;
mod chain_movement;
pub mod death_anim;
mod end_sequence;
mod goal;
mod level;
mod movement;
mod player;
mod player_chain;
mod tree;
mod vines;
mod world_text;

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
        world_text::plugin,
        tree::plugin,
        end_sequence::plugin,
    ));
}
