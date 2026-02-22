//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

use crate::ui::theme::{
    palette::{
        BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_HOVERED_BACKGROUND, BUTTON_PRESSED_BACKGROUND,
        BUTTON_TEXT, HEADER_TEXT, LABEL_TEXT, NORMAL_TEXT_COLOR,
    },
    prelude::InteractionPalette,
};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

pub fn menu_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Bundle {
    header_with_color(text, HEADER_TEXT)
}

/// Header with custom text color.
pub fn header_with_color(text: impl Into<String>, color: Color) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(color),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(24.0),
        TextColor(LABEL_TEXT),
    )
}

/// A simple text label with custom size.
pub fn label_with_size(text: impl Into<String>, size: f32) -> impl Bundle {
    label_with_size_and_color(text, size, LABEL_TEXT)
}

/// Label with custom size and color.
pub fn label_with_size_and_color(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

/// simple text lable with custom font
pub fn tutorial_label(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
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

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: px(340),
            height: px(80),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(5.0)),
            ..default()
        },
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    BorderColor::all(BUTTON_BORDER),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small_custom_font<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    font: Handle<Font>,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base_custom_font(
        text,
        action,
        Node {
            width: px(30),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        font,
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base_custom_font<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
    font: Handle<Font>,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    BorderColor::all(BUTTON_BORDER),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont {
                            font: font,
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}
