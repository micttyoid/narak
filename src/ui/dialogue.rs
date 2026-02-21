use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use crate::{
    game::level::{Level, LevelAssets, bosses::BossIntroPlaying},
    ui::theme::palette::{BACKGROUND_DARK, BUTTON_BORDER, BUTTON_TEXT},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (advance_dialogue, update_typewriter)
            .run_if(resource_exists::<DialogueQueue>.and(in_state(Level::Tutorial))),
    );
}

#[derive(Resource, Default)]
pub struct DialogueQueue {
    pub lines: Vec<String>,
    pub current_index: usize,
    // --- Typewriter Fields ---
    pub timer: Timer,
    pub visible_chars: usize,
    pub is_finished: bool,
}

impl DialogueQueue {
    pub fn new(lines: Vec<String>) -> Self {
        Self {
            lines,
            current_index: 0,
            // 0.05 is roughly 20 characters per second
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            visible_chars: 0,
            is_finished: false,
        }
    }
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
    intro_q: Query<Entity, With<BossIntroPlaying>>,
    mut time: ResMut<Time<Physics>>,
) {
    let explicit_continue = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || mouse_buttons.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();
    if explicit_continue {
        let current_line = dialogue.lines[dialogue.current_index].clone();
        if !dialogue.is_finished {
            // 1. FAST FORWARD
            dialogue.visible_chars = current_line.len();
            dialogue.is_finished = true;
            if let Ok(mut text) = text_query.single_mut() {
                text.0 = current_line.clone();
            }
        } else {
            // 2. GO TO NEXT LINE
            dialogue.current_index += 1;
            if dialogue.current_index < dialogue.lines.len() {
                dialogue.visible_chars = 0;
                dialogue.is_finished = false;
                // Clear text for next line start
                if let Ok(mut text) = text_query.single_mut() {
                    text.0 = "".to_string();
                }
            } else {
                // End dialogue (Existing logic)
                if let Ok(entity) = ui_query.single() {
                    cmd.entity(entity).despawn();
                    if let Ok(ett) = intro_q.single() {
                        cmd.entity(ett).remove::<BossIntroPlaying>();
                    }
                }
                cmd.remove_resource::<DialogueQueue>();
                time.unpause();
            }
        }
    }
}

fn update_typewriter(
    time: Res<Time>,
    mut dialogue: ResMut<DialogueQueue>,
    mut text_query: Query<&mut Text, With<DialogueUiText>>,
) {
    if dialogue.is_finished {
        return;
    }

    let current_line = dialogue.lines[dialogue.current_index].clone();

    // Tick the timer
    if dialogue.timer.tick(time.delta()).just_finished() {
        if dialogue.visible_chars < current_line.len() {
            dialogue.visible_chars += 1;

            // Update the actual UI text
            if let Ok(mut text) = text_query.single_mut() {
                text.0 = current_line.chars().take(dialogue.visible_chars).collect();
            }
        } else {
            dialogue.is_finished = true;
        }
    }
}
