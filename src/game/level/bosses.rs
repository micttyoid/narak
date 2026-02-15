use crate::{
    PausableSystems,
    game::{
        animation::AnimationAssets,
        level::enemies::{Enemy, EnemyAttack, Move, ShootingPattern},
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
pub const GATES_NAME: &str = "Gate Boss";
pub const MAYA_NAME: &str = "Eye Boss";
pub const MURA_NAME: &str = "Elephant Boss";
pub const NARAK_NAME: &str = "Son Boss";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_boss_moves, boss_teleport_system).in_set(PausableSystems),
    );
}

/// "1 boss per level, if boss gets life zero, auto move on?" "yes"
/// Do not despawn at the life zero like the other enemies
#[derive(Component)]
#[require(GameplayLifetime, Enemy)]
pub struct Boss;

fn update_boss_moves(
    time: Res<Time>,
    enemy_query: Query<(&mut LinearVelocity, &mut Enemy), With<Boss>>,
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

// boss1
#[derive(Asset, Clone, Reflect)]
pub struct GatesAssets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub attacks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
}

// boss2
#[derive(Asset, Clone, Reflect)]
pub struct MayaAssets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub attacks: Vec<Handle<AudioSource>>,
    #[dependency]
    pub damages: Vec<Handle<AudioSource>>,
    #[dependency]
    pub idle: Handle<AudioSource>,
}

// boss3
#[derive(Asset, Clone, Reflect)]
pub struct MuraAssets {
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

// boss4
#[derive(Asset, Clone, Reflect)]
pub struct NarakAssets {
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
        Enemy::new_random(1),
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.mura.enemy.clone(),
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

// boss 1 HP 6
pub fn gate_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    (
        Name::new(GATES_NAME),
        Boss,
        Enemy::new_random(6)
            .with_shooting_range(300.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Flank {
                    angle: 10.0_f32.to_radians(),
                }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
                duration: Timer::from_seconds(4.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Spread {
                    count: 5,
                    arc: 45.0_f32.to_radians(),
                }],
            }),
        AseAnimation {
            animation: Animation::tag("closed")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.gates.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, BOSS_Z_TRANSLATION),
        RigidBody::Static,
        GravityScale(0.0),
        Dominance(5), // dominates all dynamic bodies with a dominance lower than `5`.
        Collider::rectangle(50., 50.),
    )
}

// boss 2 HP 12
pub fn eye_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new(MAYA_NAME),
        Boss,
        Enemy::new_random(1)
            .with_shooting_range(300.)
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                duration: Timer::from_seconds(1.0, TimerMode::Once),
                shooting_pattern: vec![
                    ShootingPattern::Flank {
                        angle: 45.0_f32.to_radians(),
                    },
                    ShootingPattern::Straight,
                ],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                duration: Timer::from_seconds(2.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Spread {
                    count: 6,
                    arc: 90.0_f32.to_radians(),
                }],
            })
            .with_attack(EnemyAttack {
                cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                duration: Timer::from_seconds(3.0, TimerMode::Once),
                shooting_pattern: vec![ShootingPattern::Random {
                    count: 9,
                    arc: 60.0_f32.to_radians(),
                }],
            }),
        AseAnimation {
            animation: Animation::tag("idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.maya.aseprite.clone(),
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

// boss 3 HP 24
pub fn elephant_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new(MURA_NAME),
        Boss,
        Enemy::new_random(1)
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
            aseprite: anim_assets.enemies.mura.aseprite.clone(),
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

// boss 4 HP 48
pub fn son_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new(NARAK_NAME),
        Boss,
        Enemy::new_random(1)
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
                        count: 6,
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
            aseprite: anim_assets.enemies.narak.aseprite.clone(),
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
