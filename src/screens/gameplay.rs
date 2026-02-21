//! The screen state for the main gameplay.

use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_aseprite_ultra::prelude::{AseAnimation, ManualTick};

use crate::{
    AppSystems, Pause,
    audio::sound_effect,
    game::{
        level::{Level, bosses::Boss, enemies::Enemy, sfx_intro, spawn_level},
        player::*,
    },
    screens::Screen,
    ui::{menus::Menu, theme::interaction::InteractionAssets},
    utils::tiled::spawn_tiled_map,
};

/// The entity lives throug [`Screen::Gameplay`]
#[derive(Component, Default)]
pub struct GameplayLifetime;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_tiled_map, spawn_level, sfx_intro).chain(),
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
            despawn_finished_transitions.run_if(
                in_state(Screen::Gameplay)
                    .or(in_state(Screen::Loading))
                    .or(in_state(Menu::Win)),
            ),
        ),
    );
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause, cleanup));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        (
            tick_fade_in_out.in_set(AppSystems::TickTimers),
            apply_fade_in_out.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_observer(on_boss_defeated)
        .add_observer(transition_level);
}

#[derive(Component)]
pub struct LevelTransitionOverlay;

/// Top game loop. If boss is low on life, the level is set to the
/// next until the last level.
/// "1 boss per level, if boss gets life zero, auto move on?" "yes"
fn check_boss_and_player(
    mut next_menu: ResMut<NextState<Menu>>,
    query: Query<(Entity, &Enemy), With<Boss>>,
    player: Single<&Player>,
    transition_query: Query<(), With<LevelTransitionOverlay>>,
    mut cmd: Commands,
) {
    match query.single() {
        Ok((boss_entity, boss_enemy)) => {
            if boss_enemy.life == 0 && transition_query.is_empty() {
                // start screen transition splash here
                cmd.trigger(BossDefeated {
                    entity: boss_entity,
                });
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct LoadingFadeInOut {
    /// Total duration in seconds.
    pub total_duration: f32,
    /// Fade duration in seconds.
    pub fade_duration: f32,
    /// Current progress in seconds, between 0 and [`Self::total_duration`].
    pub t: f32,
}

pub const LOADING_SPLASH_DURATION_SECS: f32 = 1.2;
pub const LOADING_FADE_DURATION_SECS: f32 = 0.6;

impl LoadingFadeInOut {
    fn alpha(&self) -> f32 {
        // Normalize by duration.
        let t = (self.t / self.total_duration).clamp(0.0, 1.0);
        let fade = self.fade_duration / self.total_duration;

        // Regular trapezoid-shaped graph, flat at the top with alpha = 1.0.
        ((1.0 - (2.0 * t - 1.0).abs()) / fade).min(1.0)
    }
}

#[derive(Event)]
pub struct StartLoadNext;

pub fn tick_fade_in_out(
    time: Res<Time>,
    mut animation_query: Query<(&mut LoadingFadeInOut, Has<LevelTransitionOverlay>)>,
    mut cmd: Commands,
) {
    for (mut anim, is_exit) in &mut animation_query {
        let previously_passed = anim.t >= (anim.total_duration / 2.0);
        anim.t += time.delta_secs();
        let currently_passed = anim.t >= (anim.total_duration / 2.0);
        if is_exit && !previously_passed && currently_passed {
            cmd.trigger(StartLoadNext);
        }
    }
}

pub fn apply_fade_in_out(mut animation_query: Query<(&LoadingFadeInOut, &mut BackgroundColor)>) {
    for (anim, mut bg) in &mut animation_query {
        bg.0.set_alpha(anim.alpha());
    }
}

#[derive(EntityEvent, Copy, Clone)]
pub struct BossDefeated {
    #[event_target]
    pub entity: Entity,
}

fn on_boss_defeated(_: On<BossDefeated>, mut cmd: Commands, mut time: ResMut<Time<Physics>>) {
    time.pause();
    cmd.spawn((
        Name::new("Level Transition Overlay"),
        LevelTransitionOverlay,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.0)),
        GlobalZIndex(5),
        LoadingFadeInOut {
            total_duration: LOADING_SPLASH_DURATION_SECS,
            fade_duration: LOADING_FADE_DURATION_SECS,
            t: 0.0,
        },
    ));
}

fn transition_level(
    _: On<StartLoadNext>,
    current_level: Res<State<Level>>,
    mut next_level: ResMut<NextState<Level>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    let lev = current_level.get();
    if !lev.is_last() {
        next_level.set(lev.next());
        next_screen.set(Screen::Loading);
    } else {
        next_menu.set(Menu::Win);
    }
}

fn despawn_finished_transitions(
    mut cmd: Commands,
    query: Query<(Entity, &LoadingFadeInOut), With<LevelTransitionOverlay>>,
) {
    for (entity, anim) in &query {
        if anim.t >= anim.total_duration {
            cmd.entity(entity).despawn();
        }
    }
}
