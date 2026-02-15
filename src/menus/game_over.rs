use bevy::prelude::*;

use bevy::{ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    menus::Menu,
    screens::Screen,
    theme::{interaction::InteractionAssets, palette::BACKGROUND_DARK, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<GameOverAssets>();
    app.add_systems(
        OnEnter(Menu::GameOver),
        (spawn_game_over, start_game_over_music),
    );
}

fn spawn_game_over(mut cmd: Commands, assets: Res<InteractionAssets>) {
    cmd.spawn((
        widget::ui_root("Game Over Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::GameOver),
        children![
            (
                Name::new("Background Image"),
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image: assets.cover.clone(),
                    ..default()
                },
            ),
            (
                Node {
                    width: percent(40),
                    height: percent(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: px(20),
                    padding: UiRect::all(px(40)),
                    ..default()
                },
                BackgroundColor(BACKGROUND_DARK.with_alpha(0.6)),
                children![
                    widget::header("You Died"),
                    widget::button("Retry", retry_level),
                    widget::button("Quit to title", return_to_main),
                ],
            )
        ],
    ));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct GameOverAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for GameOverAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/sound_effects/ui/lose.ogg"),
        }
    }
}

fn start_game_over_music(mut commands: Commands, game_over_assets: Res<GameOverAssets>) {
    commands.spawn((
        Name::new("Game Over Music"),
        DespawnOnExit(Menu::GameOver),
        music(game_over_assets.music.clone()),
    ));
}

fn return_to_main(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn retry_level(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Loading);
}
