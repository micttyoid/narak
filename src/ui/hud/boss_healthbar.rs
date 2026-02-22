use bevy::prelude::*;

use crate::{
    game::level::{Level, bosses::BossPhase, enemies::Enemy},
    screens::Screen,
    ui::menus::Menu,
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

/// Ghost bar that "catches up"
#[derive(Component)]
struct GhostBarFill;

fn spawn_healthbar(mut cmd: Commands) {
    cmd.spawn((
        GlobalZIndex(1),
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(20.0),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-200.0)),
            top: Val::Percent(2.0),
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
                border_radius: BorderRadius::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.02, 0.02)),
            BorderColor::all(Color::srgb(1.0, 0.753, 0.0)),
            children![
                // 1. THE GHOST BAR (Yellow/White)
                (
                    GhostBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.9, 0.8, 0.2)), // Muted Yellow
                ),
                // 2. THE ACTUAL HEALTH BAR (Red)
                (
                    HealthBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
                    children![(
                        // 3. THE "DEPTH" OVERLAY
                        // A dark semi-transparent strip at the bottom
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(4.0),
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                    )]
                )
            ]
        )],
    ));
}

fn update_health_bar(
    time: Res<Time>,
    boss_q: Query<(&BossPhase, &Enemy)>,
    mut fill_query: Query<&mut Node, (With<HealthBarFill>, Without<GhostBarFill>)>,
    mut ghost_query: Query<&mut Node, (With<GhostBarFill>, Without<HealthBarFill>)>,
) {
    let Ok((boss, enemy)) = boss_q.single() else {
        return;
    };
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };
    let Ok(mut ghost_node) = ghost_query.single_mut() else {
        return;
    };

    let target_pct =
        (boss.current_base_hp() + enemy.life as u32) as f32 / boss.total_hp() as f32 * 100.0;

    // Snap the main bar
    fill_node.width = Val::Percent(target_pct);

    // Smoothly lerp the ghost bar down
    if let Val::Percent(current_ghost_pct) = ghost_node.width {
        if current_ghost_pct > target_pct {
            let new_pct = current_ghost_pct - (10.0 * time.delta_secs());
            ghost_node.width = Val::Percent(new_pct.max(target_pct));
        } else {
            ghost_node.width = Val::Percent(target_pct);
        }
    }
}
