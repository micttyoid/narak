pub mod bosses;
pub mod enemies;
pub mod enemy_behavior;
pub mod projectiles;

use avian2d::prelude::{Physics, PhysicsTime};

use bevy::{prelude::*, state::state::FreelyMutableState};

use crate::{
    asset_tracking::LoadResource,
    audio::{music, sound_effect},
    game::{
        animation::AnimationAssets,
        level::bosses::{phase1_boss, phase2_boss, phase3_boss, tutorial_boss},
        player::{PLAYER_Z_TRANSLATION, player},
    },
    screens::Screen,
    ui::{menus::Menu, theme::palette::LABEL_TEXT},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>()
        .init_state::<Level>()
        .add_plugins((enemy_behavior::plugin, projectiles::plugin, bosses::plugin));
}

/// GDD "pre defined multiple maps/levels(maybe 4-5?)"
/// [`Level`] exists in both [`Screen::Gameplay`] and [`Screen::Loading`]
/// When a condition meets at [`screens::gameplay::check_boss_and_player`],
/// The next is level is set, and screen is set [`Screen::Loading`].
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum Level {
    #[default]
    Tutorial,
    Phase1, // 0 to 30 / 135 HP
    Phase2, // 30 to 75 / 135 HP
    Phase3, // 75 to 135 / 135 HP
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
    pub const LAST_LEVEL: Level = Level::Phase3;

    pub fn next(&self) -> Self {
        use Level::*;
        match self {
            Tutorial => Phase1,
            Phase1 => Phase2,
            Phase2 => Phase3,
            Phase3 => Phase3,
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
            Phase1 => 6,
            Phase2 => 9,
            Phase3 => 12,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    pub ui_assets: Handle<Image>,
    #[dependency]
    pub aim_cursor: Handle<Image>,
    #[dependency]
    pub tutorial_assets: Handle<Image>,
    #[dependency]
    pub level_font: Handle<Font>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            //music: assets.load("audio/music/Fluffing A Duck.ogg"),
            music: assets.load("audio/music/Feverdream.ogg"),
            ui_assets: assets.load("textures/props/gameplay_ui.png"),
            aim_cursor: assets.load("textures/props/cursor.png"),
            tutorial_assets: assets.load("textures/props/keyboard.png"),
            level_font: assets.load("fonts/boldspixels.ttf"),
        }
    }
}

pub fn sfx_intro(
    mut commands: Commands,
    current_level: Res<State<Level>>,
    anim_assets: Res<AnimationAssets>,
) {
    use Level::*;
    match current_level.get() {
        Phase2 => {
            commands.spawn(sound_effect(anim_assets.enemies.phase2.intro.clone()));
        }
        _ => {}
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
                tutorial_boss((-30., 180.).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None), // To remove at ending such as to [`Menu::Credit`]
                    music(level_assets.music.clone()),
                ),
                (
                    Name::new("Tutorial Text"),
                    Text2d::new(
                        "Aim & Attack to Kill Enemies\n     Fulfill your Dharma".to_string()
                    ),
                    TextFont {
                        font: level_assets.level_font.clone(),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(LABEL_TEXT),
                    Transform::from_xyz(-30.0, 225.0, PLAYER_Z_TRANSLATION),
                )
            ],));
        }
        Phase1 => {
            let player_initial_transform = Vec2::new(-8.0, -211.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                // basic_enemy((66., 100.).into(), &anim_assets),
                // basic_enemy((-131., 210.).into(), &anim_assets),
                phase1_boss((38.3, 449.5).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None), // To remove at ending such as to [`Menu::Credit`]
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        Phase2 => {
            let player_initial_transform = Vec2::new(-30.0, -360.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                // eye_enemy((150., -20.).into(), &anim_assets),
                // eye_enemy((-150., -20.).into(), &anim_assets),
                phase2_boss((-36.5, 222.0).into(), &anim_assets),
                (
                    Name::new("Gameplay Music"),
                    DespawnOnExit(Menu::None),
                    music(level_assets.music.clone()),
                ),
            ],));
        }
        // Mura => {
        //     let player_initial_transform = Vec2::new(-160.0, -340.0);
        //     commands.entity(lev_entity).insert((children![
        //         player(
        //             100.0,
        //             &anim_assets,
        //             player_initial_transform,
        //             current_level.player_stats()
        //         ),
        //         snake_enemy((-186.5, -53.0).into(), &anim_assets),
        //         snake_enemy((104.5, -48.3).into(), &anim_assets),
        //         eye_enemy((57.8, -281.8).into(), &anim_assets),
        //         eye_enemy((-190.6, 120.6).into(), &anim_assets),
        //         elephant_boss((20., 330.).into(), &anim_assets),
        //         (
        //             Name::new("Gameplay Music"),
        //             DespawnOnExit(Menu::None),
        //             music(level_assets.music.clone()),
        //         ),
        //     ],));
        // }
        Phase3 => {
            let player_initial_transform = Vec2::new(-175.0, -420.0);
            commands.entity(lev_entity).insert((children![
                player(
                    100.0,
                    &anim_assets,
                    player_initial_transform,
                    current_level.player_stats()
                ),
                // narak_enemy((75.3, -333.8).into(), &anim_assets),
                // snake_enemy((-33.6, 112.2).into(), &anim_assets),
                // narak_enemy((-189.5, 282.0).into(), &anim_assets),
                // eye_enemy((152.8, 261.0).into(), &anim_assets),
                // snake_enemy((-186.5, -215.0).into(), &anim_assets),
                phase3_boss((0., 400.).into(), &anim_assets),
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
