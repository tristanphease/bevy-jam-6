use avian2d::math::Vector;

use crate::{game::death_anim::PlayerDying, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default().with_length_unit(PIXELS_PER_METER));
    app.insert_resource(Gravity(Vector::NEG_Y * 500.0));

    app.add_systems(StateFlush, Pause.on_edge(unpause_physics, pause_physics));
    app.add_systems(
        StateFlush,
        PlayerDying.on_edge(unpause_physics, pause_physics),
    );
}

const PIXELS_PER_METER: f32 = 16.0;

#[cfg_attr(feature = "native_dev", hot)]
fn unpause_physics(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.unpause();
}

#[cfg_attr(feature = "native_dev", hot)]
fn pause_physics(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.pause();
}
