use bevy::prelude::*;

mod boss_healthbar;
mod gameplay_ui;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((gameplay_ui::plugin, boss_healthbar::plugin));
}
