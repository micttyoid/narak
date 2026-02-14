use avian2d::{math::TAU, prelude::*};
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
pub struct ShootingEnemy {
    pub cooldown_timer: Timer,
    pub shooting_pattern: ShootingPattern,
    pub shooting_range: f32,
}

#[derive(Debug, Clone)]
pub enum ShootingPattern {
    Straight,
    Triple,
    Cross,
}

fn enemy_shooting_system(
    mut cmd: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut ShootingEnemy), Without<Player>>,
    anim_assets: Res<AnimationAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // No player, don't shoot
    };
    let player_pos = player_transform.translation.xy();
    for (enemy_transform, mut shooter) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation.xy();
        let distance_to_player = player_pos.distance(enemy_pos);
        if distance_to_player <= shooter.shooting_range {
            shooter.cooldown_timer.tick(time.delta());
            if shooter.cooldown_timer.just_finished() {
                let enemy_pos = enemy_transform.translation.xy();
                let enemy_radius = 12.0; // Should match enemy collider radius
                let dir = (player_pos - enemy_pos).normalize();
                let base = Dir2::new(dir).unwrap_or(Dir2::NEG_Y);
                let directions = match &shooter.shooting_pattern {
                    ShootingPattern::Straight => {
                        vec![base]
                    }
                    ShootingPattern::Triple => {
                        let rhs = dir.rotate(Vec2::from_angle(20.0));
                        let lhs = dir.rotate(Vec2::from_angle(-20.0));
                        vec![
                            base,
                            Dir2::new(rhs).unwrap_or(Dir2::Y),
                            Dir2::new(lhs).unwrap_or(Dir2::Y),
                        ]
                    }
                    ShootingPattern::Cross => {
                        let perp = dir.perp();
                        vec![
                            base,
                            Dir2::new(perp).unwrap_or(Dir2::Y),
                            Dir2::new(-perp).unwrap_or(Dir2::NEG_Y),
                            Dir2::new(-dir).unwrap_or(Dir2::Y),
                        ]
                    }
                };
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
        ShootingEnemy {
            cooldown_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            shooting_pattern: ShootingPattern::Straight,
            shooting_range: 100.0,
        },
    )
}

pub fn eye_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy::new_random(5), // GDD "Enemies to have 1-5 lives then maybe?"
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
        ShootingEnemy {
            cooldown_timer: Timer::from_seconds(4.0, TimerMode::Repeating),
            shooting_pattern: ShootingPattern::Cross,
            shooting_range: 200.0,
        },
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
        ShootingEnemy {
            cooldown_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            shooting_pattern: ShootingPattern::Triple,
            shooting_range: 100.0,
        },
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
