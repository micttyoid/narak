use avian2d::{math::TAU, prelude::*};
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use rand::{Rng, seq::IndexedRandom};

use crate::{
    PausableSystems,
    audio::sound_effect,
    game::{
        animation::AnimationAssets,
        level::{
            bosses::{
                Boss, BossIntroPlaying, PHASE_1_NAME, PHASE_2_NAME, PHASE_3_NAME,
                TUTORIAL_BOSS_NAME,
            },
            enemies::Enemy,
            projectiles::{Hostile, boss_basic_bullet, enemy_basic_bullet},
        },
        player::Player,
    },
    utils::safe_dir,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            check_enemy_death,
            update_moves,
            update_boss_moves,
            enemy_shooting_system,
            boss_teleport_system,
        )
            .in_set(PausableSystems),
    );
}

fn update_moves(
    time: Res<Time>,
    enemy_query: Query<(&mut LinearVelocity, &mut Enemy), Without<Boss>>,
) {
    let d = time.delta();
    for (mut velocity, mut enemy) in enemy_query {
        let mut is_pop = false;
        if let Some(m) = enemy.moves.last_mut() {
            match m {
                Move::UnitVelocity(v, timer) => {
                    if timer.is_finished() {
                        is_pop = true;
                        *velocity = LinearVelocity::ZERO;
                    } else {
                        timer.tick(d);
                        *velocity = *v;
                    }
                } //_ => {},
            }
        } else {
            enemy.random_linear_moves(); // refill
        }
        enemy.moves.pop_if(|_| is_pop);
    }
}

fn update_boss_moves(
    time: Res<Time>,
    enemy_query: Query<(&mut LinearVelocity, &mut Enemy), (With<Boss>, Without<BossIntroPlaying>)>,
) {
    let d = time.delta();
    for (mut velocity, mut enemy) in enemy_query {
        let mut is_pop = false;
        if let Some(m) = enemy.moves.last_mut() {
            match m {
                Move::UnitVelocity(v, timer) => {
                    if timer.is_finished() {
                        is_pop = true;
                        *velocity = LinearVelocity::ZERO;
                    } else {
                        timer.tick(d);
                        *velocity = *v;
                    }
                } //_ => {},
            }
        } else {
            enemy.random_linear_moves(); // refill
        }
        enemy.moves.pop_if(|_| is_pop);
    }
}

fn check_enemy_death(
    mut enemy_query: Query<(Entity, &Enemy, &mut AseAnimation), Without<Boss>>,
    mut events: MessageReader<AnimationEvents>,
    mut cmd: Commands,
) {
    for (entity, enemy, mut animation) in enemy_query.iter_mut() {
        if enemy.life == 0 {
            animation.animation.play("Death", AnimationRepeat::Count(0));
            // mark as dead so this runs once
            cmd.entity(entity)
                .remove::<RigidBody>()
                .remove::<Collider>()
                .remove::<GravityScale>()
                .remove::<LockedAxes>();
        }
    }
    for event in events.read() {
        match event {
            AnimationEvents::Finished(entity) => cmd.entity(*entity).despawn(),
            AnimationEvents::LoopCycleFinished(_entity) => (),
        };
    }
}

// maybe simplify this later
fn enemy_shooting_system(
    mut cmd: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut Enemy, Has<Boss>, Option<&Name>),
        (Without<Player>, Without<BossIntroPlaying>),
    >,
    anim_assets: If<Res<AnimationAssets>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player, don't shoot
    };
    let player_pos = player_transform.translation.xy();
    for (enemy_transform, mut shooter, is_boss, o_name) in enemy_query.iter_mut() {
        if shooter.attacks.is_empty() {
            continue;
        }
        let enemy_pos = enemy_transform.translation.xy();
        let distance_to_player = player_pos.distance(enemy_pos);
        if distance_to_player <= shooter.shooting_range {
            let idx = shooter.attack_idx % shooter.attacks.len();
            let current_attack = &mut shooter.attacks[idx];
            // check duration of current attack - reset if finished
            current_attack.duration.tick(time.delta());
            if current_attack.duration.is_finished() {
                current_attack.duration.reset();
                shooter.attack_idx = (idx + 1) % shooter.attacks.len();
                continue;
            }
            // shoot on cooldown
            current_attack.cooldown_timer.tick(time.delta());
            if current_attack.cooldown_timer.just_finished() {
                let enemy_pos = enemy_transform.translation.xy();
                let enemy_radius = 12.0; // Should match enemy collider radius
                let dir = (player_pos - enemy_pos).normalize();
                let mut directions = Vec::new();
                for pattern in &current_attack.shooting_pattern {
                    directions.extend(get_shooting_patterns(dir, pattern, current_attack));
                }
                for direction in directions {
                    // Generate a random color for each bullet
                    let mut rng = rand::rng();
                    let random_color = Color::srgb(
                        rng.random_range(0.3..=1.0),
                        rng.random_range(0.3..=1.0),
                        rng.random_range(0.3..=1.0),
                    );
                    if is_boss {
                        if let Some(name) = o_name {
                            match name.as_str() {
                                TUTORIAL_BOSS_NAME => {
                                    cmd.spawn(sound_effect(anim_assets.enemies.throw.clone()))
                                }
                                PHASE_1_NAME => cmd.spawn(sound_effect(
                                    anim_assets
                                        .enemies
                                        .phase1
                                        .attacks
                                        .choose(&mut rand::rng())
                                        .unwrap()
                                        .clone(),
                                )),
                                PHASE_2_NAME => cmd.spawn(sound_effect(
                                    anim_assets
                                        .enemies
                                        .phase2
                                        .attacks
                                        .choose(&mut rand::rng())
                                        .unwrap()
                                        .clone(),
                                )),
                                PHASE_3_NAME => cmd.spawn(sound_effect(
                                    anim_assets
                                        .enemies
                                        .phase3
                                        .attacks
                                        .choose(&mut rand::rng())
                                        .unwrap()
                                        .clone(),
                                )),
                                _ => {
                                    panic!("unknown boss")
                                }
                            };
                        }
                        cmd.spawn(boss_basic_bullet::<Hostile>(
                            enemy_pos,
                            direction,
                            enemy_radius,
                            &anim_assets,
                            random_color,
                        ));
                    } else {
                        cmd.spawn(sound_effect(anim_assets.enemies.throw.clone()));
                        cmd.spawn(enemy_basic_bullet::<Hostile>(
                            enemy_pos,
                            direction,
                            enemy_radius,
                            &anim_assets,
                            random_color,
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct EnemyAttack {
    pub cooldown_timer: Timer,
    pub duration: Timer,
    pub shooting_pattern: Vec<ShootingPattern>,
}

#[derive(Debug, Clone)]
pub enum ShootingPattern {
    Straight,
    Spread { count: usize, arc: f32 },
    Ring { count: usize },
    Flank { angle: f32 },
    Random { count: usize, arc: f32 },
    Sweep { arc: f32, clockwise: bool },
}

/// Shooting Patterns
fn get_shooting_patterns(
    dir: Vec2,
    pattern: &ShootingPattern,
    current_attack: &EnemyAttack,
) -> Vec<Dir2> {
    let base_angle = dir.to_angle();
    match pattern {
        ShootingPattern::Straight => vec![safe_dir(dir)],
        ShootingPattern::Spread { count, arc } => {
            if *count <= 1 {
                return vec![safe_dir(dir)];
            }
            let mut dirs = Vec::with_capacity(*count);
            let half_arc = arc / 2.0;
            // The step size between each bullet
            let step = arc / (*count as f32 - 1.0);

            for i in 0..*count {
                // Calculate offset: start from -half_arc and add step
                let angle_offset = -half_arc + (step * i as f32);
                let new_dir = Vec2::from_angle(base_angle + angle_offset);
                dirs.push(safe_dir(new_dir));
            }
            dirs
        }
        ShootingPattern::Ring { count } => {
            let mut dirs = Vec::with_capacity(*count);
            let step = TAU / *count as f32;

            for i in 0..*count {
                let angle = base_angle + (step * i as f32);
                dirs.push(safe_dir(Vec2::from_angle(angle)));
            }
            dirs
        }
        ShootingPattern::Flank { angle } => {
            vec![
                safe_dir(Vec2::from_angle(base_angle - angle)),
                safe_dir(Vec2::from_angle(base_angle + angle)),
            ]
        }
        ShootingPattern::Random { count, arc } => {
            let mut rng = rand::rng();
            let mut dirs = Vec::with_capacity(*count);
            let half_arc = arc / 2.0;
            for _ in 0..*count {
                // Random offset between -half and +half
                let offset = rng.random_range(-half_arc..=half_arc);
                dirs.push(safe_dir(Vec2::from_angle(base_angle + offset)));
            }
            dirs
        }
        ShootingPattern::Sweep { arc, clockwise } => {
            // store Sweep delay from cooldown and total_sweeps by dividing it by duration
            let duration_secs = current_attack.duration.duration().as_secs_f32();
            let cooldown_secs = current_attack.cooldown_timer.duration().as_secs_f32();
            let total_sweeps = (duration_secs / cooldown_secs).max(1.0) as usize;

            // Derive which shot we're on from elapsed time
            let elapsed_secs = current_attack.duration.elapsed().as_secs_f32();
            let shot_index =
                ((elapsed_secs / cooldown_secs) as usize).min(total_sweeps.saturating_sub(1));

            let step = if total_sweeps > 1 {
                arc / (total_sweeps - 1) as f32
            } else {
                0.0
            };

            // false: left → right (+arc/2 start sweeping right)
            // true:  right → left (-arc/2 start sweeping left)
            let angle_offset = if *clockwise {
                (arc / 2.0) - step * shot_index as f32
            } else {
                -(arc / 2.0) + step * shot_index as f32
            };
            let sweep_dir = Vec2::from_angle(base_angle + angle_offset);
            vec![safe_dir(sweep_dir)]
        }
    }
}

// It'll do so for "unit" time
#[derive(Clone)]
pub enum Move {
    UnitVelocity(LinearVelocity, Timer),
    // UnitWeirdMotion,
    // UnitDance,
    // UnitPathfinding,
}

#[derive(Component)]
pub struct TeleportAbility {
    pub positions: Vec<Vec2>,
    pub timer: Timer,
    pub current_index: usize,
}

fn boss_teleport_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut TeleportAbility)>) {
    for (mut transform, mut teleport) in query.iter_mut() {
        teleport.timer.tick(time.delta());
        if teleport.timer.just_finished() {
            if !teleport.positions.is_empty() {
                let next_pos = &teleport.positions[teleport.current_index];
                transform.translation.x = next_pos.x;
                transform.translation.y = next_pos.y;
                teleport.current_index = (teleport.current_index + 1) % teleport.positions.len();
            }
        }
    }
}
