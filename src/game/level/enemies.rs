use avian2d::{
    math::{PI, TAU},
    prelude::*,
};
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use rand::Rng;

use crate::{
    PausableSystems,
    game::{
        animation::*,
        level::projectiles::{Hostile, enemy_basic_bullet},
        movement::*,
        player::{PLAYER_Z_TRANSLATION, Player},
    },
    screens::gameplay::GameplayLifetime,
    utils::safe_dir,
};

pub const ENEMY_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (check_enemy_death, update_moves, enemy_shooting_system).in_set(PausableSystems),
    );
}

/// "1 boss per level, if boss gets life zero, auto move on?" "yes"
/// Do not despawn at the life zero like the other enemies
#[derive(Component)]
#[require(GameplayLifetime, Enemy)]
pub struct Boss;

#[derive(Component)]
#[require(GameplayLifetime, Collider)]
pub struct Enemy {
    pub life: usize,
    pub moves: Vec<Move>,
    pub attacks: Vec<EnemyAttack>,
    pub shooting_range: f32,
    pub attack_idx: usize,
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

impl Default for Enemy {
    fn default() -> Self {
        Self {
            life: 1, // GDD "Enemies to have 1-5 lives then maybe?"
            moves: Vec::<Move>::new(),
            attacks: Vec::<EnemyAttack>::new(),
            shooting_range: 100.0,
            attack_idx: 0,
        }
    }
}

impl Enemy {
    pub const RANDOM_MAX_SPEED: f32 = 65.0; // 101
    pub const RANDOM_MIN_SPEED: f32 = 15.0;
    pub const RANDOM_MAX_TIME: f32 = 2.0;
    pub const RANDOM_MIN_TIME: f32 = 0.5;
    pub const RANDOM_MAX_MOVES: usize = 10;
    pub const RANDOM_MIN_MOVES: usize = 2;

    pub fn new_random(life: usize) -> Self {
        Self {
            life,
            moves: Self::get_random_linear_moves(),
            attacks: Vec::new(),
            shooting_range: 100.0,
            attack_idx: 0,
        }
    }

    pub fn random_linear_moves(&mut self) {
        self.moves.append(&mut Self::get_random_linear_moves());
    }

    pub fn get_random_linear_moves() -> Vec<Move> {
        let mut rng = rand::rng();
        let n = rng.random_range(Self::RANDOM_MIN_MOVES..=Self::RANDOM_MAX_MOVES);
        Self::get_random_linear_n_moves(n)
    }

    pub fn get_random_linear_n_moves(n: usize) -> Vec<Move> {
        let mut ms = Vec::<Move>::new();
        (0..n).for_each(|_| ms.push(Self::get_random_move_unit_velocity()));
        ms
    }

    pub fn get_random_move_unit_velocity() -> Move {
        let mut rng = rand::rng();
        let mag: f32 = rng.random_range(Self::RANDOM_MIN_SPEED..=Self::RANDOM_MAX_SPEED);
        let ang: f32 = rng.random_range(0.0..TAU); // Repetitive bikeshed at the math channel
        Move::UnitVelocity(
            LinearVelocity(mag * Vec2::new(ang.cos(), ang.sin())),
            Timer::from_seconds(
                rng.random_range(Self::RANDOM_MIN_TIME..=Self::RANDOM_MAX_TIME),
                TimerMode::Once,
            ),
        )
    }

    pub fn with_attack(mut self, attack: EnemyAttack) -> Self {
        self.attacks.push(attack);
        self
    }
    pub fn with_shooting_range(mut self, range: f32) -> Self {
        self.shooting_range = range;
        self
    }
}

#[derive(Asset, Clone, Reflect)]
pub struct EnemyAssets {
    pub aseprite: Handle<Aseprite>,
    pub eye_enemy: Handle<Aseprite>,
    pub boss1: Handle<Aseprite>,
    pub boss2: Handle<Aseprite>,
    pub boss3: Handle<Aseprite>,
    pub boss4: Handle<Aseprite>,
    pub bullet: Handle<Image>,
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
}

// maybe simplify this later
fn enemy_shooting_system(
    mut cmd: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Enemy), Without<Player>>,
    anim_assets: Res<AnimationAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player, don't shoot
    };
    let player_pos = player_transform.translation.xy();
    for (enemy_transform, mut shooter) in enemy_query.iter_mut() {
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
                    directions.extend(get_shooting_patterns(dir, pattern));
                }
                for direction in directions {
                    cmd.spawn(enemy_basic_bullet::<Hostile>(
                        enemy_pos,
                        direction,
                        enemy_radius,
                        &anim_assets,
                    ));
                }
            }
        }
    }
}

/// Shooting Patterns
fn get_shooting_patterns(dir: Vec2, pattern: &ShootingPattern) -> Vec<Dir2> {
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
    }
}

/// An example of an enemy
pub fn basic_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy::new_random(5), // GDD "Enemies to have 1-5 lives then maybe?"
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_enemy_collision_radius),
    )
}

pub fn eye_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy::new_random(5) // GDD "Enemies to have 1-5 lives then maybe?"
            .with_shooting_range(300.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Flank {
                        angle: 45.0_f32.to_radians(),
                    },
                    ShootingPattern::Straight,
                ],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                duration: Timer::from_seconds(5.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Ring { count: 9 }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Random {
                    count: 9,
                    arc: 90.0_f32.to_radians(),
                }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.eye_enemy.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_enemy_collision_radius),
    )
}

// boss 1
pub fn gate_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    (
        Name::new("Gate Boss"),
        Boss,
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("closed")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.boss1.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Static,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::rectangle(50., 50.),
    )
}

// boss 2
pub fn eye_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Eye Boss"),
        Boss,
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.boss2.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// boss 3
pub fn elephant_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Elephant Boss"),
        Boss,
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.boss3.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// boss 4
pub fn son_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Son Boss"),
        Boss,
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.boss4.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

#[allow(dead_code)]
pub fn basic_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Boss"),
        Boss,
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(2.0),
            aseprite: anim_assets.enemies.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// It'll do so for "unit" time
#[derive(Clone)]
pub enum Move {
    UnitVelocity(LinearVelocity, Timer),
    // UnitWeirdMotion,
    // UnitDance,
    // UnitPathfinding,
}
