use crate::{
    menu::{Menu, MenuRoot},
    prelude::*,
    screen::{Screen, fade::fade_out},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(StateFlush, Menu::GameOver.on_enter(spawn_gameover_menu));
}

fn spawn_gameover_menu(mut commands: Commands, menu_root: Res<MenuRoot>) {
    commands
        .entity(menu_root.ui)
        .with_child(widget::body(children![
            widget::header("[b]You won?"),
            widget::column_of_buttons(children![widget::wide_button(
                "Back to title screen",
                quit_to_title
            ),])
        ]));
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.spawn(fade_out(Screen::Title));
}
