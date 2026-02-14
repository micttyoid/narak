use bevy::prelude::*;

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{interaction::InteractionAssets, palette::BACKGROUND_DARK, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::GameOver), spawn_game_over);
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

fn return_to_main(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn retry_level(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Loading);
}
