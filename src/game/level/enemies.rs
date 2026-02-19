use crate::{
    game::{
        animation::*,
        level::{
            bosses::{GatesAssets, MayaAssets, MuraAssets, NarakAssets},
            enemy_behavior::{EnemyAttack, Move, ShootingPattern},
        },
        movement::ScreenWrap,
        player::PLAYER_Z_TRANSLATION,
    },
    screens::gameplay::GameplayLifetime,
};
use avian2d::{math::TAU, prelude::*};
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::{
    Animation, AnimationDirection, AnimationRepeat, AseAnimation, Aseprite,
};
use rand::Rng;

pub const ENEMY_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

#[derive(Component)]
#[require(GameplayLifetime, Collider)]
pub struct Enemy {
    pub life: usize,
    pub moves: Vec<Move>,
    pub attacks: Vec<EnemyAttack>,
    pub shooting_range: f32,
    pub attack_idx: usize,
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
    pub seedlng_aseprite: Handle<Aseprite>,
    pub eye_enemy: EyeEnemyAssets,
    pub gates: GatesAssets, // boss1
    pub maya: MayaAssets,   // boss2
    pub mura: MuraAssets,   // boss3
    pub narak: NarakAssets, // boss4
    pub bullet: Handle<Image>,
    #[dependency]
    pub throw: Handle<AudioSource>,
}

#[derive(Asset, Clone, Reflect)]
pub struct EyeEnemyAssets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
}

/// An example of an enemy (Lv1 Basic Enemy)
pub fn basic_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy::new_random(3)
            .with_shooting_range(400.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Straight],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Flank {
                    angle: 22.5_f32.to_radians(),
                }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.seedlng_aseprite.clone(),
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

// Lv 2 Eye Enemy
pub fn eye_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Eye Enemy"),
        Enemy::new_random(6) // GDD "Enemies to have 1-5 lives then maybe?"
            .with_shooting_range(300.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(2.8, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Ring { count: 5 }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                duration: Timer::from_seconds(0.6, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Straight],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.eye_enemy.aseprite.clone(),
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

// Lv 3 Snake Enemy
pub fn snake_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Snake Enemy"),
        Enemy::new_random(9)
            .with_shooting_range(250.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Ring { count: 8 }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Spread {
                    count: 5,
                    arc: 60.0_f32.to_radians(), // Shotgun spread
                }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.mura.enemy.clone(),
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

// Lv 4 Narak Enemy
pub fn narak_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Narak Enemy"),
        Enemy::new_random(12)
            .with_shooting_range(400.)
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
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Ring { count: 4 },
                    ShootingPattern::Straight,
                ],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.narak.enemy.clone(),
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
