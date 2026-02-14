//! The game's menus and transitions between them.

mod credits;
mod game_over;
mod main;
mod pause;
mod settings;
mod win;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        credits::plugin,
        main::plugin,
        settings::plugin,
        pause::plugin,
        game_over::plugin,
        win::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
    GameOver,
    Win,
}
