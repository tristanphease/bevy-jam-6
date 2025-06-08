use crate::menu::Menu;
use crate::prelude::*;
use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(LoadingState::new(Screen::Title.bevy()));

    app.add_systems(
        StateFlush,
        Screen::Title.on_enter(
            (Menu::Main.enter(), Menu::acquire).chain(),
        ),
    );
}
