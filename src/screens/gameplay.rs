//! The screen state for the main gameplay.

use std::fs::exists;

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_aseprite_ultra::prelude::{AseAnimation, ManualTick};

use crate::{
    Pause,
    audio::sound_effect,
    game::{
        level::{
            Level,
            enemies::{Boss, Enemy},
            spawn_level,
        },
        player::*,
    },
    menus::Menu,
    screens::Screen,
    theme::interaction::InteractionAssets,
    utils::tiled::spawn_tiled_map,
};

/// The entity lives throug [`Screen::Gameplay`]
#[derive(Component, Default)]
pub struct GameplayLifetime;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_tiled_map, spawn_level).chain(),
    );

    app.add_systems(
        Update,
        (
            // Top-level game loop
            (check_boss_and_player).run_if(in_state(Screen::Gameplay).and(in_state(Menu::None))),
            // Toggle pause on key press.
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
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause, cleanup));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );
}

/// Top game loop. If boss is low on life, the level is set to the
/// next until the last level.
/// "1 boss per level, if boss gets life zero, auto move on?" "yes"
fn check_boss_and_player(
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_level: ResMut<NextState<Level>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut time: ResMut<Time<Physics>>,
    current_level: Res<State<Level>>,
    query: Query<(Entity, &Enemy), With<Boss>>,
    player: Single<&Player>,
) {
    match query.single() {
        Ok((_, boss_enemy)) => {
            if boss_enemy.life == 0 {
                time.pause();
                next_pause.set(Pause(true));
                let lev = current_level.get();
                if lev.is_last() {
                    next_menu.set(Menu::Win);
                } else {
                    next_level.set(lev.next());
                    next_screen.set(Screen::Loading);
                }
            }
            if (*player).life == 0 {
                // TODO: lost
                next_menu.set(Menu::GameOver);
            }
        }
        Err(_) => {
            panic!("No boss found at the current level");
        }
    }
}

fn unpause(
    mut commands: Commands,
    mut next_pause: ResMut<NextState<Pause>>,
    interaction_assets: If<Res<InteractionAssets>>,
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
    commands.spawn(sound_effect(interaction_assets.pause.clone()));
}

fn pause(
    mut commands: Commands,
    mut next_pause: ResMut<NextState<Pause>>,
    interaction_assets: If<Res<InteractionAssets>>,
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
    commands.spawn(sound_effect(interaction_assets.pause.clone()));
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

fn cleanup(mut commands: Commands, mut query: Query<Entity, With<GameplayLifetime>>) {
    query
        .iter_mut()
        .for_each(|entity| commands.entity(entity).despawn());
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
