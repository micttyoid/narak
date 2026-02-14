use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::{Animation, AnimationDirection, AnimationRepeat, AseAnimation};

use crate::{
    PausableSystems,
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
        (update_cools, update_projectiles, restore_ammo).in_set(PausableSystems),
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

/// Something like the `collision_layers` but ECS
#[derive(Component, Default, Debug)]
pub struct Friendly;

/// Something like the `collision_layers` but ECS
#[derive(Component, Default, Debug)]
pub struct Hostile;

/// Define how projectile is resolved beside hit
/// Not gonna use enumset
#[derive(Debug, PartialEq, Eq)]
pub enum Due {
    Lifespan(Timer),
    BounceDown(usize), // bounce is counted down
}

/// thrower radius: radius of the thrower
/// TODO: visual using AnimationAssets
///
/// **Example: player**
/// ```ignore
/// commands.spawn(basic_projectile::<Friendly>(
///     xy,
///     direction,
///     PLAYER_COLLIDER_RADIUS,
///     &anim_assets,
/// ));
/// ```
///
/// **Example: enemy**
/// ```ignore
/// commands.spawn(basic_projectile::<Hostile>(
///     xy,
///     direction,
///     PLAYER_COLLIDER_RADIUS,
///     &anim_assets,
/// ));
/// ```
pub fn player_chakra<HostilityComponent: Component + Default>(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    thrower_height: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let player_projectile_collider_radius: f32 = 2.;
    let speed: f32 = 300.0;
    // If the collider is not alway centered with respect to the entity this could be this or vice versa(diameter of thrower):
    //             diameter of projectile        extent_in_direction of Thrower
    // (1.0e-3) -------------------------------|---------------------------------- Thrower
    //    ^ tolerance
    // Calculate how far the capsule extends in the firing direction
    let extent_in_direction = (thrower_height + thrower_radius);
    // TODO: needs to be fixed according to player capsule collidor
    let new_xy =
        (player_projectile_collider_radius + (extent_in_direction) + 1.0e-3) * direction + xy;
    let chakram_projectile_life: f32 = 4.0; // seconds
    (
        Name::new("Chakra"),
        Projectile {
            direction,
            dues: vec![
                Due::BounceDown(5),
                Due::Lifespan(Timer::from_seconds(
                    chakram_projectile_life,
                    TimerMode::Once,
                )),
            ],
        },
        HostilityComponent::default(),
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
        Collider::circle(player_projectile_collider_radius),
        Restitution::new(2.0),
        Friction::new(0.0),
    )
}

pub fn enemy_basic_bullet<HostilityComponent: Component + Default>(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let lifespan_projectile_collider_radius: f32 = 2.;
    let projectile_life: f32 = 3.0; // seconds
    let speed: f32 = 200.0;

    let new_xy = (lifespan_projectile_collider_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Enemy Basic Projectile"),
        Projectile {
            direction,
            dues: vec![
                Due::BounceDown(3),
                Due::Lifespan(Timer::from_seconds(projectile_life, TimerMode::Once)),
            ],
        },
        HostilityComponent::default(),
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        Sprite {
            image: anim_assets.enemies.bullet.clone(),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(lifespan_projectile_collider_radius),
        Restitution::new(1.5),
        Friction::new(0.0),
    )
}

#[allow(dead_code)]
pub fn basic_projectile<HostilityComponent: Component + Default>(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let basic_projectile_collider_radius: f32 = 2.;
    let speed: f32 = 300.0;
    //          radius of proj             radius of thrower
    //    pr ----------------------|---------------------------------- Thrower
    //    ^ spawned with room

    let new_xy = (basic_projectile_collider_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Basic Projectile"),
        Projectile {
            direction,
            dues: vec![],
        },
        HostilityComponent::default(),
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
        Collider::circle(basic_projectile_collider_radius),
        Restitution::new(1.0),
    )
}

/// Example of projectile that's gone when it bounces more than certain time
/// TODO: visual using AnimationAssets
#[allow(dead_code)]
pub fn bounce_down_projectile<HostilityComponent: Component + Default>(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let bounce_down_projectile_collider_radius: f32 = 2.;
    let speed: f32 = 500.0;
    let new_xy =
        (bounce_down_projectile_collider_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Bounce Down Projectile"),
        Projectile {
            direction,
            dues: vec![Due::BounceDown(5)],
        },
        HostilityComponent::default(),
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        //Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(bounce_down_projectile_collider_radius),
        Restitution::new(1.0),
    )
}

/// Example of projectile that has lifespan
/// TODO: visual using AnimationAssets
#[allow(dead_code)]
pub fn lifespan_projectile<HostilityComponent: Component + Default>(
    xy: Vec2,
    direction: Dir2,
    thrower_radius: f32,
    anim_assets: &AnimationAssets,
) -> impl Bundle {
    let lifespan_projectile_collider_radius: f32 = 2.;
    let lifespan_projectile_life: f32 = 5.0; // seconds
    let speed: f32 = 500.0;

    let new_xy = (lifespan_projectile_collider_radius + thrower_radius + 1.0e-3) * direction + xy;
    (
        Name::new("Life Span Projectile"),
        Projectile {
            direction,
            dues: vec![Due::Lifespan(Timer::from_seconds(
                lifespan_projectile_life,
                TimerMode::Once,
            ))],
        },
        HostilityComponent::default(),
        LinearVelocity(speed * direction.as_vec2()),
        LinearDamping(0.0),
        Sprite {
            image: anim_assets.enemies.bullet.clone(),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        ScreenWrap,
        LockedAxes::new().lock_rotation(),
        Transform::from_xyz(new_xy.x, new_xy.y, PROJECTILE_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(lifespan_projectile_collider_radius),
        Restitution::new(1.0),
    )
}

// Project due process except what's collision based
fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    projectile_query: Query<(Entity, &mut Projectile)>,
) {
    let mut despawned = Vec::<Entity>::new();
    for (proj_entity, mut projectile) in projectile_query {
        for due in projectile.dues.iter_mut() {
            use Due::*;
            match due {
                Lifespan(timer) => {
                    if timer.is_finished() {
                        despawned.push(proj_entity);
                        break;
                    } else {
                        timer.tick(time.delta());
                    }
                }
                BounceDown(_count) => { /* nothing: this  handled at [`on_collision`] */ }
            }
        }
    }
    despawned.iter().for_each(|&e| commands.entity(e).despawn());
}

fn restore_ammo(mut player: Single<&mut Player>, mut removed: RemovedComponents<Friendly>) {
    if !removed.is_empty() {
        let mut p = player.into_inner();
        p.increment_ammo(removed.len());
    }
    removed.clear();
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
#[allow(dead_code)]
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
