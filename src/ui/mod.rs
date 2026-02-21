use bevy::prelude::*;

pub mod dialogue;
mod hud;
pub mod menus;
pub mod theme;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((hud::plugin, menus::plugin, theme::plugin, dialogue::plugin));
}
