use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation};
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
    app.add_systems(
        Update,
        (update_sources, update_projectiles).in_set(PausableSystems),
    );
    app.add_systems(FixedUpdate, (update_cools).in_set(PausableSystems));
}

// TODO:
// Start throwing by two cases:
// - A source gets to the desired location
// - A source timer/timly-trigger is due/activated
fn update_sources() {}

// TODO:
// If the projectile has no relation with a source it should have some
// sort of way it dies(despawn) itself beside collision with player
// possibly by:
// - timer
// - out of bound (game screen) (not if the project has circular pattern?)
// - task complete: ex. creating another source it was supposed to create
fn update_projectiles() {}

/// The chakra, bullet, ...
/// [`Player`] can throw. [`Mob`] can throw. or throw [`Source`] instead
/// Game should run smoothly roughly 1000 projectiles: https://youtu.be/AY7QEEnSGVU
#[derive(Component, Debug)]
#[require(GameplayLifetime, Collider, CollisionEventsEnabled)]
pub struct Projectile {
    pub direction: Dir2,
}

impl Default for Projectile {
    fn default() -> Self {
        Self {
            direction: Dir2::NEG_Y,
        }
    }
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
    let speed: f32 = 300.0;
    //          radius of proj             radius of thrower
    //    pr ----------------------|---------------------------------- Thrower
    //    ^ spawned with room

    let new_xy = (basic_projectile_collision_radius + thrower_radius + 1.0e-5) * direction + xy;
    (
        Name::new("Basic Projectile"),
        Projectile { direction },
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        AseAnimation {
            animation: Animation::tag("Spin")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.player.chakram.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_projectile_collision_radius),
        Restitution::new(1.0),
    )
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
