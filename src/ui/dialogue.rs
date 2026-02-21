use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use crate::{
    game::level::{Level, LevelAssets},
    ui::theme::palette::{BACKGROUND_DARK, BUTTON_BORDER, BUTTON_TEXT},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (advance_dialogue).run_if(resource_exists::<DialogueQueue>.and(in_state(Level::Tutorial))),
    );
}

#[derive(Resource, Default)]
pub struct DialogueQueue {
    pub lines: Vec<String>,
    pub current_index: usize,
}

#[derive(Component)]
pub struct DialogueUi;

#[derive(Component)]
pub struct DialogueUiText;

pub fn spawn_dialogue_ui(cmd: &mut Commands, assets: &LevelAssets, initial_text: &str) {
    cmd.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(80.0),
            height: Val::Percent(30.0),
            bottom: Val::Percent(30.0),
            left: Val::Percent(10.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        DialogueUi,
        BackgroundColor(BACKGROUND_DARK.with_alpha(0.8)),
        BorderColor::all(BUTTON_BORDER),
        DespawnOnExit(Level::Tutorial),
        children![(
            DialogueUiText,
            Text::new(initial_text),
            TextFont {
                font: assets.level_font.clone(),
                font_size: 25.0,
                ..default()
            },
            TextColor(BUTTON_TEXT),
        )],
    ));
}

fn advance_dialogue(
    mut cmd: Commands,
    mut dialogue: ResMut<DialogueQueue>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut text_query: Query<&mut Text, With<DialogueUiText>>,
    ui_query: Query<Entity, With<DialogueUi>>,
    mut time: ResMut<Time<Physics>>,
) {
    let explicit_continue = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse_buttons.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    if explicit_continue {
        dialogue.current_index += 1;

        if dialogue.current_index < dialogue.lines.len() {
            // Update text to next line
            if let Ok(mut text) = text_query.single_mut() {
                text.0 = dialogue.lines[dialogue.current_index].clone();
            }
        } else {
            // End of conversation: Despawn UI and Resume Game
            if let Ok(entity) = ui_query.single() {
                cmd.entity(entity).despawn();
            }
            cmd.remove_resource::<DialogueQueue>();

            time.unpause();
        }
    }
}
