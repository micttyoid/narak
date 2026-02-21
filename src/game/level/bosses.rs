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
    pub const PHASE_1_HP: u32 = 2; // change to 30
    pub const PHASE_2_HP: u32 = 3; // change to 45
    pub const PHASE_3_HP: u32 = 4; // change to 60

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
    pub aseprite: Handle<Aseprite>,
    pub enemy: Handle<Aseprite>,
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
    pub enemy: Handle<Aseprite>,
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
            aseprite: anim_assets.enemies.phase2.enemy.clone(),
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
            .with_shooting_range(250.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Ring { count: 9 }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Random {
                    count: 9,
                    arc: 10.0_f32.to_radians(),
                }],
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
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Dynamic,
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
                duration: Timer::from_seconds(1.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Flank {
                        angle: 45.0_f32.to_radians(),
                    },
                    ShootingPattern::Straight,
                ],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Spread {
                    count: 4,
                    arc: 90.0_f32.to_radians(),
                }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Random {
                    count: 5,
                    arc: 60.0_f32.to_radians(),
                }],
            }),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.phase2.aseprite.clone(),
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

// boss 3 HP 60
pub fn phase3_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 24.;
    (
        Name::new(PHASE_3_NAME),
        Boss,
        BossPhase::for_phase(3),
        BossIntroPlaying,
        Enemy::new_random(BossPhase::PHASE_3_HP as usize)
            .with_shooting_range(250.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                duration: Timer::from_seconds(5.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Ring { count: 7 }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Random {
                    count: 6,
                    arc: 90.0_f32.to_radians(),
                }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Spread {
                        count: 4,
                        arc: 90.0_f32.to_radians(),
                    },
                    ShootingPattern::Ring { count: 4 },
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
                Vec2::new(xy.x, xy.y),
                Vec2::new(200.0, 100.0),
                //Vec2::new(250.0, 60.0),
                Vec2::new(224.0, -345.3),
                Vec2::new(-244.1, -93.3),
                Vec2::new(-160.3, 133.5),
            ],
            timer: Timer::from_seconds(20.0, TimerMode::Repeating),
            current_index: 0,
        },
    )
}

#[derive(Component)]
pub struct BossIntroPlaying;
