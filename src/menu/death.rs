use crate::{
    menu::{Menu, MenuRoot},
    prelude::*,
    screen::{Screen, fade::fade_out},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(StateFlush, Menu::Death.on_enter(spawn_death_menu));
}

fn spawn_death_menu(mut commands: Commands, menu_root: Res<MenuRoot>) {
    commands
        .entity(menu_root.ui)
        .with_child(widget::body(children![
            widget::header("[b]RIP"),
            widget::column_of_buttons(children![
                widget::wide_button("Return to last checkpoint", last_checkpoint),
                widget::wide_button("Quit to title screen", quit_to_title),
            ])
        ]));
}

fn last_checkpoint(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.spawn(fade_out(Screen::Gameplay));
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.spawn(fade_out(Screen::Title));
}
