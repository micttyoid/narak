use crate::{
    game::{
        animation::AnimationAssets,
        level::{
            enemies::Enemy,
            enemy_behavior::{EnemyAttack, ShootingPattern, TeleportAbility},
        },
        movement::ScreenWrap,
        player::PLAYER_Z_TRANSLATION,
    },
    screens::gameplay::GameplayLifetime,
};
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

pub const BOSS_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;
pub const TUTORIAL_BOSS_NAME: &str = "Tutorial Boss";
pub const PHASE_1_NAME: &str = "Phase 1 Boss";
pub const PHASE_2_NAME: &str = "Phase 2 Boss";
pub const PHASE_3_NAME: &str = "Phase 3 Boss";

/// "1 boss per level, if boss gets life zero, auto move on?" "yes"
/// Do not despawn at the life zero like the other enemies
#[derive(Component)]
#[require(GameplayLifetime, Enemy)]
pub struct Boss;

#[derive(Component, Reflect)]
pub struct BossPhase {
    pub current_phase: u8, // 1, 2, or 3
    pub max_hp: u32,       // 30, 45, or 60
}

impl BossPhase {
    pub const PHASE_1_HP: u32 = 20; // change to 30
    pub const PHASE_2_HP: u32 = 30; // change to 45
    pub const PHASE_3_HP: u32 = 40; // change to 60

    pub fn for_phase(phase: u8) -> Self {
        let max_hp = match phase {
            1 => Self::PHASE_1_HP,
            2 => Self::PHASE_2_HP,
            3 => Self::PHASE_3_HP,
            _ => panic!("Invalid boss phase: {}", phase),
        };
        Self {
            current_phase: phase,
            max_hp,
        }
    }
    pub fn total_hp(&self) -> u32 {
        Self::PHASE_1_HP + Self::PHASE_2_HP + Self::PHASE_3_HP
    }
    pub fn current_base_hp(&self) -> u32 {
        match self.current_phase {
            1 => Self::PHASE_2_HP + Self::PHASE_3_HP,
            2 => Self::PHASE_3_HP,
            3 => 0,
            _ => panic!("Invalid boss phase: {}", self.current_phase),
        }
    }
}

// phase 1
#[derive(Asset, Clone, Reflect)]
pub struct Phase1Assets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub attacks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
    #[dependency]
    pub idle: Handle<AudioSource>,
}

// phase 2
#[derive(Asset, Clone, Reflect)]
pub struct Phase2Assets {
    #[dependency]
    pub attacks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
    #[dependency]
    pub grunts: Vec<Handle<AudioSource>>,
    #[dependency]
    pub intro: Handle<AudioSource>,
}

// phase 3
#[derive(Asset, Clone, Reflect)]
pub struct Phase3Assets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub attacks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
    #[dependency]
    pub death: Handle<AudioSource>,
}

// Boss Lv 0 HP 3
pub fn tutorial_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new(TUTORIAL_BOSS_NAME),
        Boss,
        BossIntroPlaying,
        Enemy::new_random(3)
            .with_shooting_range(300.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.5, TimerMode::Repeating), // delay between each sweep shot
                duration: Timer::from_seconds(1.0, TimerMode::Once), // total duration of attack
                shooting_pattern: vec![ShootingPattern::Sweep {
                    arc: 90.0_f32.to_radians(), // angle of the arc in radians
                    clockwise: true,            // direction
                }],
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
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// // boss 1 HP 6
// pub fn gate_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
//     (
//         Name::new(GATES_NAME),
//         Boss,
//         Enemy::new_random(6)
//             .with_shooting_range(300.)
//             .with_attack(EnemyAttack {
//                 cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
//                 duration: Timer::from_seconds(4.0, TimerMode::Once),
//                 shooting_pattern: vec![ShootingPattern::Flank {
//                     angle: 10.0_f32.to_radians(),
//                 }],
//             })
//             .with_attack(EnemyAttack {
//                 cooldown_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
//                 duration: Timer::from_seconds(4.0, TimerMode::Once),
//                 shooting_pattern: vec![ShootingPattern::Spread {
//                     count: 5,
//                     arc: 45.0_f32.to_radians(),
//                 }],
//             }),
//         AseAnimation {
//             animation: Animation::tag("closed")
//                 .with_repeat(AnimationRepeat::Loop)
//                 .with_direction(AnimationDirection::Forward)
//                 .with_speed(1.0),
//             aseprite: anim_assets.enemies.gates.aseprite.clone(),
//         },
//         Sprite::default(),
//         ScreenWrap,
//         LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
//         Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
//         RigidBody::Static,
//         GravityScale(0.0),
//         Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
//         Collider::rectangle(50., 50.),
//     )
// }

// phase 1 HP 30
pub fn phase1_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 32.;
    (
        Name::new(PHASE_1_NAME),
        Boss,
        BossPhase::for_phase(1),
        BossIntroPlaying,
        Enemy::new_random(BossPhase::PHASE_1_HP as usize)
            .with_shooting_range(300.)
            // Attack 1: A clean 3-shot spread
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
                duration: Timer::from_seconds(2.4, TimerMode::Once), // 3 bursts
                shooting_pattern: vec![ShootingPattern::Spread {
                    count: 5,
                    arc: 90.0_f32.to_radians(),
                }],
            })
            // Attack 2: A spaced-out ring to dodge through
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once), // 1 burst
                shooting_pattern: vec![ShootingPattern::Ring { count: 8 }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.phase1.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Static,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// phase 2 HP +45
pub fn phase2_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 32.;
    (
        Name::new(PHASE_2_NAME),
        Boss,
        BossPhase::for_phase(2),
        BossIntroPlaying,
        Enemy::new_random(BossPhase::PHASE_2_HP as usize) // change to 45
            .with_shooting_range(400.)
            // Attack 1: Fast sweeping motion forcing the player to run
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                duration: Timer::from_seconds(1.5, TimerMode::Once), // 10 shots in the sweep
                shooting_pattern: vec![ShootingPattern::Sweep {
                    arc: 120.0_f32.to_radians(),
                    clockwise: true,
                }],
            })
            // Attack 2: Wide Spread mixed with Random suppression fire
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once), // 3 bursts
                shooting_pattern: vec![
                    ShootingPattern::Spread {
                        count: 4,
                        arc: 75.0_f32.to_radians(),
                    },
                    ShootingPattern::Random {
                        count: 3,
                        arc: 45.0_f32.to_radians(),
                    },
                ],
            })
            // Attack 3: Fast sweeping motion forcing the player to run in other direction
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                duration: Timer::from_seconds(1.5, TimerMode::Once), // 10 shots in the sweep
                shooting_pattern: vec![ShootingPattern::Sweep {
                    arc: 120.0_f32.to_radians(),
                    clockwise: false,
                }],
            })
            // Attack 4: Faster, denser ring from Phase 1
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                duration: Timer::from_seconds(1.5, TimerMode::Once), // 1 burst
                shooting_pattern: vec![ShootingPattern::Ring { count: 10 }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.phase1.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation().lock_translation_y(),
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
    )
}

// boss 3 HP 60
pub fn phase3_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 24.;
    (
        Name::new(PHASE_3_NAME),
        Boss,
        BossPhase::for_phase(3),
        BossIntroPlaying,
        Enemy::new_random(BossPhase::PHASE_3_HP as usize)
            .with_shooting_range(350.)
            // Attack 1: The Pincer (Straight + Flank) - Punishes standing still
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.6, TimerMode::Repeating),
                duration: Timer::from_seconds(2.4, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Straight,
                    ShootingPattern::Flank {
                        angle: 35.0_f32.to_radians(),
                    },
                ],
            })
            // Attack 2: Bullet Hell Chaos (Dense Ring + Random)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Ring { count: 12 },
                    ShootingPattern::Random {
                        count: 5,
                        arc: 90.0_f32.to_radians(),
                    },
                ],
            })
            // Attack 3: The Wall (Sweep + Tight Spread)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once), // 8 sweep steps
                shooting_pattern: vec![
                    ShootingPattern::Sweep {
                        arc: 90.0_f32.to_radians(),
                        clockwise: false,
                    },
                    ShootingPattern::Spread {
                        count: 2,
                        arc: 15.0_f32.to_radians(),
                    },
                ],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.phase3.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::circle(basic_enemy_collision_radius),
        TeleportAbility {
            positions: vec![
                Vec2::new(xy.x, 110.0),
                Vec2::new(245.0, 58.0),
                Vec2::new(-29.3, 0.0),
                Vec2::new(-220.0, -116.3),
                Vec2::new(-226.0, 58.0),
                Vec2::new(176.1, -120.3),
            ],
            timer: Timer::from_seconds(20.0, TimerMode::Repeating),
            current_index: 0,
        },
    )
}

#[derive(Component)]
pub struct BossIntroPlaying;

#[derive(Component)]
pub struct BossIntroTimer(pub Timer);
