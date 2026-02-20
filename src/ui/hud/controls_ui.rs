use bevy::prelude::*;

use crate::{
    game::level::LevelAssets,
    screens::Screen,
    ui::{menus::Menu, theme::widget::tutorial_label},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_tutorial_ui);
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
        DespawnOnEnter(Menu::Win),
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
