use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_aseprite_ultra::prelude::*;

use crate::{
    PausableSystems,
    game::{
        movement::*,
        animation::*,
        player::PLAYER_Z_TRANSLATION,
    },
    screens::gameplay::GameplayLifetime,    
};

pub const ENEMY_Z_TRANSLATION: f32 = PLAYER_Z_TRANSLATION;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_enemies,
        ).in_set(PausableSystems),
    );
}

fn update_enemies(
    mut commands: Commands,
) {

}

#[derive(Component)]
#[require(GameplayLifetime)]
pub struct Enemy {
    life: usize,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            life: 1, // "Enemies to have 1-5 lives then maybe?""
        }
    }
}

#[derive(Asset, Clone, Reflect)]
pub struct EnemyAssets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,    
}

/// An example of an enemy
pub fn basic_enemy(xy: Vec2, anim_assets: &AnimationAssets) -> impl Bundle {
    let basic_enemy_collision_radius: f32 = 12.;
    (
        Name::new("Basic Enemy"),
        Enemy { life: 5 },
        EnemyAnimation {
            state: EnemyAnimationState::default(),
            direction: EnemyDirection::default(),
        },
        AseAnimation {
            animation: Animation::tag("walk-up")
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
    )
}
