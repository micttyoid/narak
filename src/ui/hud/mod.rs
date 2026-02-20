use bevy::prelude::*;

mod boss_healthbar;
mod controls_ui;
mod player_stats;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player_stats::plugin,
        boss_healthbar::plugin,
        controls_ui::plugin,
    ));
}
