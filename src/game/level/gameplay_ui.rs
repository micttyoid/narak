use std::borrow::Cow;

use bevy::{
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage},
};

use crate::{
    game::{level::LevelAssets, player::Player},
    menus::Menu,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_gameplay_ui)
        .add_systems(
            Update,
            update_gameplay_stats
                .run_if(in_state(Screen::Gameplay))
                .run_if(in_state(Menu::None)),
        );
}

/// Marker component for the UI container
#[derive(Component)]
struct GameplayUiContainer;

/// Marker component for heart icons
#[derive(Component)]
struct HeartIcon {
    index: usize,
}

/// Marker component for ammo icons
#[derive(Component)]
struct AmmoIcon {
    index: usize,
}

/// Atlas indices for the UI sprites (4x4 grid of 32x32 pixels)
#[derive(Clone, Copy)]
enum UiIconAtlas {
    HeartFilled = 0,
    HeartEmpty = 1,
    AmmoFilled = 2,
    AmmoEmpty = 3,
}

const ICON_SIZE: f32 = 32.0;
const ICON_SPACING: f32 = 8.0;

/// Spawns the gameplay UI showing player health and ammo and switches mouse cursor to aim
pub fn spawn_gameplay_ui(
    mut cmd: Commands,
    assets: Res<LevelAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<&Player>,
    window: Single<Entity, With<Window>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 2, 2, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    let (max_hearts, max_ammo) = if let Ok(player) = player_query.single() {
        (player.max_life, player.max_ammo)
    } else {
        (3, 3)
    };

    cmd.spawn((
        Name::new("Gameplay UI"),
        GameplayUiContainer,
        GlobalZIndex(1),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(50.0),
            top: Val::Px(50.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        DespawnOnExit(Screen::Gameplay),
        DespawnOnEnter(Menu::Credits),
    ))
    .with_children(|ui| {
        ui.spawn(stat_container("Hearts Container"))
            .with_children(|ui| {
                for i in 0..max_hearts {
                    ui.spawn((
                        HeartIcon { index: i },
                        ImageNode::from_atlas_image(
                            assets.ui_assets.clone(),
                            TextureAtlas {
                                layout: layout_handle.clone(),
                                index: UiIconAtlas::HeartFilled as usize, // Default to full
                            },
                        ),
                        Node {
                            width: Val::Px(ICON_SIZE),
                            height: Val::Px(ICON_SIZE),
                            ..default()
                        },
                    ));
                }
            });
        ui.spawn(stat_container("Ammo Container"))
            .with_children(|ui| {
                for i in 0..max_ammo {
                    ui.spawn((
                        Name::new(format!("Ammo {}", i)),
                        AmmoIcon { index: i },
                        ImageNode::from_atlas_image(
                            assets.ui_assets.clone(),
                            TextureAtlas {
                                layout: layout_handle.clone(),
                                index: UiIconAtlas::AmmoFilled as usize, // Default to full
                            },
                        ),
                        Node {
                            width: Val::Px(ICON_SIZE),
                            height: Val::Px(ICON_SIZE),
                            ..default()
                        },
                    ));
                }
            });
    });

    // spawning aim cursor
    cmd.entity(*window)
        .insert((CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
            handle: assets.aim_cursor.clone(),
            ..default()
        })),));
}

pub fn stat_container(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(ICON_SPACING),
            align_items: AlignItems::Center,
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// Updates heart and ammo icons when player stats change
fn update_gameplay_stats(
    player_query: Query<&Player, Changed<Player>>,
    mut heart_query: Query<(&HeartIcon, &mut ImageNode), (With<HeartIcon>, Without<AmmoIcon>)>,
    mut ammo_query: Query<(&AmmoIcon, &mut ImageNode), (With<AmmoIcon>, Without<HeartIcon>)>,
) {
    if let Ok(player) = player_query.single() {
        // Update Hearts
        for (icon, mut image) in heart_query.iter_mut() {
            if let Some(atlas) = &mut image.texture_atlas {
                if icon.index < player.life {
                    atlas.index = UiIconAtlas::HeartFilled as usize;
                } else {
                    atlas.index = UiIconAtlas::HeartEmpty as usize;
                }
            }
        }

        // Update Ammo
        for (icon, mut image) in ammo_query.iter_mut() {
            if let Some(atlas) = &mut image.texture_atlas {
                if icon.index < player.ammo {
                    atlas.index = UiIconAtlas::AmmoFilled as usize;
                } else {
                    atlas.index = UiIconAtlas::AmmoEmpty as usize;
                }
            }
        }
    }
}
