pub mod enemies;
pub mod gameplay_ui;
pub mod projectiles;

use avian2d::prelude::{Physics, PhysicsTime};

use bevy::{prelude::*, state::state::FreelyMutableState};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::{
        animation::AnimationAssets,
        level::enemies::{
            basic_boss, basic_enemy, elephant_boss, eye_boss, eye_enemy, gate_boss, son_boss,
        },
        player::player,
    },
    menus::Menu,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>()
        .init_state::<Level>()
        .add_plugins((enemies::plugin, projectiles::plugin, gameplay_ui::plugin));
}

/// GDD "pre defined multiple maps/levels(maybe 4-5?)"
/// TODO: Please name the levels according to the concept! ;o
/// [`Level`] exists in both [`Screen::Gameplay`] and [`Screen::Loading`]
/// When a condition meets at [`screens::gameplay::check_boss_and_player`],
/// The next is level is set, and screen is set [`Screen::Loading`].
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum Level {
    #[default]
    Tutorial,
    Gates,
    Maya,
    Mura,
    Narak,
}

impl SubStates for Level {
    type SourceStates = Option<Screen>;

    fn should_exist(sources: Option<Screen>) -> Option<Self> {
        match sources {
            Some(Screen::Gameplay) => Some(Self::default()),
            Some(Screen::Loading) => Some(Self::default()),
            _ => None,
        }
    }
}

impl States for Level {
    const DEPENDENCY_DEPTH: usize = <Level as SubStates>::SourceStates::SET_DEPENDENCY_DEPTH + 1;
}

impl FreelyMutableState for Level {}

impl Level {
    pub const LAST_LEVEL: Level = Level::Narak;

    pub fn next(&self) -> Self {
        use Level::*;
        match self {
            Tutorial => Gates,
            Gates => Maya,
            Maya => Mura,
            Mura => Narak,
            Narak => panic!("No more next level: It is the last level"),
        }
    }

    pub fn is_last(&self) -> bool {
        if *self == Self::LAST_LEVEL {
            true
        } else {
            false
        }
    }

    pub fn player_stats(&self) -> usize {
        use Level::*;
        match self {
            Tutorial => 3,
            Gates => 6,
            Maya => 9,
            Mura => 12,
            Narak => 15,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    ui_assets: Handle<Image>,
    #[dependency]
    aim_cursor: Handle<Image>,
    #[dependency]
    pub tutorial_assets: Handle<Image>,
    #[dependency]
    pub level_font: Handle<Font>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            ui_assets: assets.load("textures/props/gameplay_ui.png"),
            aim_cursor: assets.load("textures/props/cursor.png"),
            tutorial_assets: assets.load("textures/props/keyboard.png"),
            level_font: assets.load("fonts/boldspixels.ttf"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    current_level: Res<State<Level>>,
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    anim_assets: Res<AnimationAssets>,
    mut time: ResMut<Time<Physics>>,
) {
    let lev_entity = commands
        .spawn((
            Name::new("Level"),
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();
    use Level::*;

    match current_level.get() {
        Tutorial => {
            let player_initial_transform = Vec2::new(-30.0, 0.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                basic_boss((-30., 220.).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None), // To remove at ending such as to [`Menu::Credit`]
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        Gates => {
            let player_initial_transform = Vec2::new(20.0, -100.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                eye_enemy((80., 100.).into(), &anim_assets),
                eye_enemy((-80., 200.).into(), &anim_assets),
                gate_boss((0., 370.).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None), // To remove at ending such as to [`Menu::Credit`]
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        Maya => {
            let player_initial_transform = Vec2::new(-30.0, -360.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                eye_enemy((150., -20.).into(), &anim_assets),
                eye_enemy((-150., -20.).into(), &anim_assets),
                eye_boss((-30.0, 240.0).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None),
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        Mura => {
            let player_initial_transform = Vec2::new(-160.0, -340.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                basic_enemy((-70., -60.).into(), &anim_assets),
                basic_enemy((-60., -50.).into(), &anim_assets),
                elephant_boss((20., 330.).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None),
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        Narak => {
            let player_initial_transform = Vec2::new(0.0, 0.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                basic_enemy((-70., 20.).into(), &anim_assets),
                basic_enemy((-60., 0.).into(), &anim_assets),
                son_boss((140., 40.).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None),
                    music(level_assets.music.clone()),
                ),
            ],));
        }
    }
    time.unpause();
}
