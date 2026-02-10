use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

use crate::{
    PausableSystems,
    game::{animation::*, movement::*, player::PLAYER_Z_TRANSLATION},
    screens::gameplay::GameplayLifetime,
    utils::collisions_layers::GameLayer,
};

pub const ENEMY_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (check_enemy_death).in_set(PausableSystems));
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
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            life: 1, // GDD "Enemies to have 1-5 lives then maybe?"
        }
    }
}

#[derive(Asset, Clone, Reflect)]
pub struct EnemyAssets {
    pub aseprite: Handle<Aseprite>,
}

/// An example of an enemy
pub fn basic_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy { life: 5 }, // GDD "Enemies to have 1-5 lives then maybe?"
        AseAnimation {
            animation: Animation::tag("Idle")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(1.0),
            aseprite: anim_assets.enemies.aseprite.clone(),
        },
        Sprite::default(),
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(xy.x, xy.y, ENEMY_Z_TRANSLATION),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(basic_enemy_collision_radius),
        CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Walls, GameLayer::Player, GameLayer::FriendlyProj],
        ),
    )
}

pub fn basic_boss(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Boss"),
        Boss,
        Enemy { life: 1 },
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
        Collider::circle(basic_enemy_collision_radius),
        CollisionLayers::new(
            GameLayer::Enemy,
            [GameLayer::Walls, GameLayer::Player, GameLayer::FriendlyProj],
        ),
    )
}

fn check_enemy_death(
    mut enemy_query: Query<(Entity, &Enemy, &mut AseAnimation)>,
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
