use std::borrow::Cow;

use bevy::{
    prelude::*,
    window::{CursorIcon, CustomCursor, CustomCursorImage},
};

use crate::{
    game::{
        level::{Level, LevelAssets},
        player::Player,
    },
    menus::Menu,
    screens::Screen,
    theme::palette::LABEL_TEXT,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_gameplay_ui, spawn_tutorial_ui),
    )
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
    window: Single<Entity, With<Window>>,
    current_level: Res<State<Level>>,
) {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 2, 2, None, None);
    let layout_handle = texture_atlas_layouts.add(layout);

    let stats = current_level.player_stats();

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
        DespawnOnEnter(Menu::Credits), // credits menu switches to gameplay - hence this needs to be handled
    ))
    .with_children(|ui| {
        ui.spawn(stat_container("Hearts Container"))
            .with_children(|ui| {
                for i in 0..stats {
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
                for i in 0..stats {
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

/// Marker component for tutorial UI
#[derive(Component)]
struct TutorialUi;

fn spawn_tutorial_ui(
    mut cmd: Commands,
    assets: Res<LevelAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Initialize an empty layout for the 192x96 image
    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(192, 96));

    // 1. Attack: Space (Row 2, Col 1-4)
    let attack_idx = layout.add_texture(URect::new(0, 16, 64, 32));
    // 2. Movement: WASD (Row 3, Col 1-4)
    let w_idx = layout.add_texture(URect::new(0, 32, 16, 48));
    let a_idx = layout.add_texture(URect::new(16, 32, 32, 48));
    let s_idx = layout.add_texture(URect::new(32, 32, 48, 48));
    let d_idx = layout.add_texture(URect::new(48, 32, 64, 48));
    // 3. Mouse Hover: (Rows 5-6, Cols 9-10)
    let mouse_hover_idx = layout.add_texture(URect::new(128, 64, 160, 96));
    let layout_handle = texture_atlas_layouts.add(layout);
    cmd.spawn((
        Name::new("Tutorial UI"),
        TutorialUi,
        GlobalZIndex(1),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(30.0),
            bottom: Val::Px(30.0),
            flex_direction: FlexDirection::Row,
            padding: UiRect::all(Val::Px(16.0)),
            column_gap: Val::Px(16.0),
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        DespawnOnExit(Screen::Gameplay),
        DespawnOnEnter(Menu::Credits),
        children![
            (
                // Mouse Aim
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                children![
                    (
                        ImageNode::from_atlas_image(
                            assets.tutorial_assets.clone(),
                            TextureAtlas {
                                layout: layout_handle.clone(),
                                index: mouse_hover_idx,
                            },
                        ),
                        Node {
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            ..default()
                        },
                    ),
                    tutorial_label("Aim", assets.level_font.clone())
                ]
            ),
            ((
                // Attack
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                children![
                    (
                        ImageNode::from_atlas_image(
                            assets.tutorial_assets.clone(),
                            TextureAtlas {
                                layout: layout_handle.clone(),
                                index: attack_idx
                            },
                        ),
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                    ),
                    tutorial_label("Attack", assets.level_font.clone())
                ]
            )),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(8.0),
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                },
                children![
                    // Container for the keys arranged in a typical layout
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        children![
                            // Top row: W
                            (
                                ImageNode::from_atlas_image(
                                    assets.tutorial_assets.clone(),
                                    TextureAtlas {
                                        layout: layout_handle.clone(),
                                        index: w_idx,
                                    },
                                ),
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                            ),
                            // Bottom row: A, S, D
                            (
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    ..default()
                                },
                                children![
                                    (
                                        ImageNode::from_atlas_image(
                                            assets.tutorial_assets.clone(),
                                            TextureAtlas {
                                                layout: layout_handle.clone(),
                                                index: a_idx,
                                            },
                                        ),
                                        Node {
                                            width: Val::Px(16.0),
                                            height: Val::Px(16.0),
                                            ..default()
                                        },
                                    ),
                                    (
                                        ImageNode::from_atlas_image(
                                            assets.tutorial_assets.clone(),
                                            TextureAtlas {
                                                layout: layout_handle.clone(),
                                                index: s_idx,
                                            },
                                        ),
                                        Node {
                                            width: Val::Px(16.0),
                                            height: Val::Px(16.0),
                                            ..default()
                                        },
                                    ),
                                    (
                                        ImageNode::from_atlas_image(
                                            assets.tutorial_assets.clone(),
                                            TextureAtlas {
                                                layout: layout_handle.clone(),
                                                index: d_idx,
                                            },
                                        ),
                                        Node {
                                            width: Val::Px(16.0),
                                            height: Val::Px(16.0),
                                            ..default()
                                        },
                                    ),
                                ],
                            ),
                        ],
                    ),
                    tutorial_label("Movement", assets.level_font.clone())
                ]
            )
        ],
    ));
}

/// Tutorial text label.
fn tutorial_label(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont {
            font: font,
            font_size: 20.0,
            ..default()
        },
        TextColor(LABEL_TEXT),
    )
}
