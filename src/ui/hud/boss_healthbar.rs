use bevy::prelude::*;

use crate::{
    game::level::{Level, bosses::BossPhase, enemies::Enemy},
    screens::Screen,
    ui::{menus::Menu, theme::palette::BUTTON_BORDER},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (
            spawn_healthbar.run_if(in_state(Level::Phase1)),
            spawn_healthbar.run_if(in_state(Level::Phase2)),
            spawn_healthbar.run_if(in_state(Level::Phase3)),
        ),
    )
    .add_systems(Update, update_health_bar.run_if(in_state(Screen::Gameplay)));
}

/// Marks the health bar fill node
#[derive(Component)]
struct HealthBarFill;

fn spawn_healthbar(mut cmd: Commands) {
    cmd.spawn((
        GlobalZIndex(1),
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(10.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-200.0)),
            top: Val::Percent(5.0),
            padding: UiRect::top(Val::Px(12.0)),
            ..default()
        },
        DespawnOnExit(Screen::Gameplay),
        DespawnOnEnter(Screen::Loading),
        DespawnOnEnter(Menu::Win), // for when credits menu is opened at the end of gameplay
        children![(
            Node {
                width: Val::Px(400.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
            BorderColor::all(BUTTON_BORDER),
            children![(
                HealthBarFill,
                Node {
                    width: Val::Percent(100.0), // starts full
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
            )]
        )],
    ));
}

fn update_health_bar(
    boss_q: Query<(&BossPhase, &Enemy)>,
    mut fill_query: Query<&mut Node, With<HealthBarFill>>,
) {
    let Ok((boss, enemy)) = boss_q.single() else {
        return;
    };
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };

    let pct = (boss.current_base_hp() + enemy.life as u32) as f32 / boss.total_hp() as f32;
    fill_node.width = Val::Percent(pct * 100.0);
}
