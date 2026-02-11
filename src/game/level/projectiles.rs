use avian2d::prelude::*;
use bevy::prelude::*;
use std::ops::{Deref, DerefMut};

use crate::{
    AppSystems, PausableSystems,
    game::{
        animation::*,
        level::enemies::{basic_boss, basic_enemy},
        movement::*,
        player::*,
    },
    screens::gameplay::GameplayLifetime,
};

pub const PROJECTILE_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;
pub const SOURCE_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_sources.in_set(PausableSystems));
    app.add_systems(
        FixedUpdate,
        (update_cools, update_projectiles).in_set(PausableSystems),
    );
}

// TODO:
// Start throwing by two cases:
// - A source gets to the desired location
// - A source timer/timly-trigger is due/activated
fn update_sources() {}

/// The chakra, bullet, ...
/// [`Player`] can throw. [`Mob`] can throw. or throw [`Source`] instead
/// Game should run smoothly roughly 1000 projectiles: https://youtu.be/AY7QEEnSGVU
#[derive(Component, Debug)]
#[require(GameplayLifetime, Collider, CollisionEventsEnabled)]
pub struct Projectile {
    pub direction: Dir2,
    pub dues: Vec<Due>,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            direction: Dir2::NEG_Y,
            dues: vec![],
        }
    }
}

/// Define how projectile is resolved beside hit
/// Not gonna use enumset
#[derive(Debug, PartialEq, Eq)]
pub enum Due {
    Lifespan(Timer),
    BounceDown(usize), // bounce is counted down
}

/// thrower radius: radius of the thrower
/// TODO: visual using AnimationAssets
pub fn basic_projectile(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let basic_projectile_collision_radius: f32 = 2.;
    let speed: f32 = 500.0;
    //          radius of proj             radius of thrower
    //    pr ----------------------|---------------------------------- Thrower
    //    ^ spawned with room

    let new_xy = (basic_projectile_collision_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Basic Projectile"),
        Projectile {
            direction,
            dues: vec![],
        },
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        //Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_projectile_collision_radius),
        Restitution::new(1.0),
    )
}

/// Example of projectile that's gone when it bounces more than certain time
/// TODO: visual using AnimationAssets
pub fn bounce_down_projectile(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let bounce_down_projectile_collision_radius: f32 = 2.;
    let speed: f32 = 500.0;
    let new_xy =
        (bounce_down_projectile_collision_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Bounce Down Projectile"),
        Projectile {
            direction,
            dues: vec![Due::BounceDown(5)],
        },
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        //Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(bounce_down_projectile_collision_radius),
        Restitution::new(1.0),
    )
}

/// Example of projectile that has lifespan
/// TODO: visual using AnimationAssets
pub fn lifespan_projectile(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let lifespan_projectile_collision_radius: f32 = 2.;
    let lifespan_projectile_life: f32 = 5.0; // seconds
    let speed: f32 = 500.0;

    let new_xy = (lifespan_projectile_collision_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Life Span Projectile"),
        Projectile {
            direction,
            dues: vec![Due::Lifespan(Timer::from_seconds(
                lifespan_projectile_life,
                TimerMode::Once,
            ))],
        },
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        //Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(lifespan_projectile_collision_radius),
        Restitution::new(1.0),
    )
}

// Project due process except what's collision based
fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<Entity, With<Player>>,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
) {
    for (proj_entity, mut projectile) in projectile_query {
        for mut due in projectile.dues.iter_mut() {
            use Due::*;
            match due {
                Lifespan(timer) => {
                    timer.tick(time.delta());
                    if timer.is_finished() {
                        commands.entity(proj_entity).despawn();
                    }
                }
                BounceDown(count) => { /* nothing */ }
            }
        }
    }
}

/// Player Projectile Cooldown - limit the projectiles player can have thrown at a time
/// It is for now used only for projectile, but not limited to it.
#[derive(Component, Debug, Clone, Default)]
#[require(GameplayLifetime)]
pub struct Cool(pub Timer);

impl Cool {
    pub fn new(duration: f32) -> Self {
        Self {
            0: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// NOTE: For more control, use virtual time
fn update_cools(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<Entity, With<Player>>,
    mut cool_query: Query<(Entity, &mut Cool), Without<Player>>,
) {
    for (_, mut cool) in cool_query.iter_mut() {
        cool.0.tick(time.delta());
    }
    for (cool_entity, cool) in cool_query.iter() {
        if cool.0.is_finished() {
            commands.entity(*player).insert_if_new(cool.clone());
            commands.entity(cool_entity).despawn();
        }
    }
}

/// The source where the projectiles spread out from
/// This is to make projectile pattern that's spread out from a source like
/// in the game Touhou.
/// Not necessarily colliding
/// [`Player`] can throw. [`Mob`] can throw.
#[derive(Component, Debug)]
#[require(GameplayLifetime)]
pub struct Source {
    pub direction: Dir2,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            direction: Dir2::NEG_Y,
        }
    }
}
