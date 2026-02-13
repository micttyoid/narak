//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
    menus::Menu,
    screens::Screen,
    theme::{interaction::InteractionAssets, palette::BACKGROUND_DARK, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, assets: Res<InteractionAssets>) {
    commands.spawn((
        widget::menu_root("Main Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
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
                Name::new("Menu Sidebar"),
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
                #[cfg(not(target_family = "wasm"))]
                children![
                    widget::header("Narak"),
                    widget::button("Play", enter_loading_or_gameplay_screen),
                    widget::button("Settings", open_settings_menu),
                    widget::button("Credits", open_credits_menu),
                    widget::button("Exit", exit_app),
                ],
                #[cfg(target_family = "wasm")]
                children![
                    widget::header("Narak"),
                    widget::button("Play", enter_loading_or_gameplay_screen),
                    widget::button("Settings", open_settings_menu),
                    widget::button("Credits", open_credits_menu),
                ],
            ),
        ],
    ));
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
