use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        animation::*,
        level::enemies::{basic_boss, basic_enemy},
        movement::*,
        player::*,
    },
    screens::gameplay::GameplayLifetime,
    utils::collisions_layers::GameLayer,
};

pub const PROJECTILE_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (update_sources, update_projectiles).in_set(PausableSystems),
    );
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
#[derive(Component)]
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

/// The source where the projectiles spread out from
/// This is to make projectile pattern that's spread out from a source like
/// in the game touhou.
/// [`Player`] can throw. [`Mob`] can throw.
#[derive(Component)]
#[require(GameplayLifetime)]
pub struct Source;

// TODO: anim
pub fn basic_projectile(xy: Vec2, direction: Dir2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_projectile_collision_radius: f32 = 2.;
    let speed: f32 = 500.0;
    //    pr ----- | ---------- P
    //    ^ spawned with room
    let new_xy =
        (basic_projectile_collision_radius + PLAYER_COLLIDER_RADIUS + 1.0e-5) * direction + xy;
    (
        Name::new("Basic Projectile"),
        Projectile { direction },
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        //Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        //Transform::from_xyz(xy.x, xy.y, PROJECTILE_Z_TRANSLATION),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_projectile_collision_radius),
        Restitution::new(1.0),
        CollisionLayers::new(
            GameLayer::FriendlyProj,
            [GameLayer::Walls, GameLayer::Enemy],
        ),
    )
}
