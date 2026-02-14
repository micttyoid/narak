//! Stuffs that haven't been on the three main directories in the original template
pub mod cam;
pub mod collisions_layers;
pub mod tiled; // Named to be distinct from bevy::camera

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((tiled::plugin, cam::plugin));
}

pub fn safe_dir(v: Vec2) -> Dir2 {
    Dir2::new(v).unwrap_or(Dir2::Y)
}
