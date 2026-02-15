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
    app.load_resource::<WinAssets>();
    app.add_systems(OnEnter(Menu::Win), (spawn_win, start_win_music));
}

fn spawn_win(mut cmd: Commands, assets: Res<InteractionAssets>) {
    cmd.spawn((
        widget::ui_root("All cleared"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Win),
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
                    widget::header("Narak Slayed"),
                    widget::button("Credits", show_credits),
                    widget::button("Quit to title", return_to_main),
                ],
            )
        ],
    ));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct WinAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for WinAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/sound_effects/ui/win.ogg"),
        }
    }
}

fn start_win_music(mut commands: Commands, win_assets: Res<WinAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        DespawnOnExit(Menu::Win),
        music(win_assets.music.clone()),
    ));
}

fn return_to_main(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn show_credits(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}
