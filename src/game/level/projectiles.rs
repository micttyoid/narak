use bevy::prelude::*;
//use avian2d::prelude::*;
use crate::{
    screens::gameplay::GameplayLifetime,
    AppSystems, PausableSystems,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_sources,
            update_projectiles,
        ).in_set(PausableSystems),
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
#[require(GameplayLifetime)]
pub struct Projectile;

/// The source where the projectiles spread out from
/// This is to make projectile pattern that's spread out from a source like 
/// in the game touhou.
/// [`Player`] can throw. [`Mob`] can throw.
#[derive(Component)]
#[require(GameplayLifetime)]
pub struct Source;