//! The screen state for the main gameplay.

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_aseprite_ultra::prelude::{AseAnimation, ManualTick};

use crate::{
    Pause, game::level::spawn_level, menus::Menu, screens::Screen, utils::tiled::spawn_tiled_map,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_tiled_map::<2>, spawn_level).chain(),
    );

    // Toggle pause on key press.
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape))),
            ),
            close_menu.run_if(
                in_state(Screen::Gameplay)
                    .and(not(in_state(Menu::None)))
                    .and(input_just_pressed(KeyCode::KeyP)),
            ),
        ),
    );
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );
}

fn unpause(
    mut next_pause: ResMut<NextState<Pause>>,
    mut time: ResMut<Time<Physics>>,
    mut cmd: Commands,
    query: Query<Entity, With<AseAnimation>>,
) {
    next_pause.set(Pause(false));
    // unpause physics time
    time.unpause();
    // Remove ManualTick to let the Aseprite animation library take over again
    for entity in &query {
        cmd.entity(entity).remove::<ManualTick>();
    }
}

fn pause(
    mut next_pause: ResMut<NextState<Pause>>,
    mut time: ResMut<Time<Physics>>,
    mut cmd: Commands,
    query: Query<Entity, With<AseAnimation>>,
) {
    next_pause.set(Pause(true));
    // pause physics time
    time.pause();
    // Add ManualTick to all animations when pausing (stops Aseprite animations)
    for entity in &query {
        cmd.entity(entity).insert(ManualTick);
    }
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        DespawnOnExit(Pause(true)),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
