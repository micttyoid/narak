use bevy::prelude::*;

use avian2d::prelude::*;
use bevy_aseprite_ultra::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        animation::{AnimationAssets, PlayerAnimation, PlayerAnimationState, PlayerDirection},
        movement::{MovementController, ScreenWrap},
    },
    utils::collisions_layers::GameLayer,
};

pub const PLAYER_Z_TRANSLATION: f32 = 100.;
pub const PLAYER_COLLIDER_RADIUS: f32 = 12.;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<AnimationAssets>();

    // Record directional input as movement controls.

    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[require(Collider)]
#[reflect(Component)]
pub struct Player {
    pub life: usize,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            life: 3, // "3 lives on player?"
        }
    }
}

/// The player character.
pub fn player(max_speed: f32, anim_assets: &AnimationAssets) -> impl Bundle {
    (
        Name::new("Player"),
        Player::default(),
        PlayerAnimation {
            state: PlayerAnimationState::default(),
            direction: PlayerDirection::default(),
        },
        AseAnimation {
            animation: Animation::tag("walk-up")
                .with_repeat(AnimationRepeat::Loop)
                .with_direction(AnimationDirection::Forward)
                .with_speed(2.0),
            aseprite: anim_assets.player.aseprite.clone(),
        },
        Sprite::default(),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        LockedAxes::new().lock_rotation(), // To be resolved with later kinematic solution
        Transform::from_xyz(0., 0., PLAYER_Z_TRANSLATION),
        // TODO: possibly kinematic later that should update `movement::apply_movement` along
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::circle(PLAYER_COLLIDER_RADIUS),
        CollisionLayers::new(
            GameLayer::Player,
            [
                GameLayer::Walls,
                GameLayer::Enemy,
                GameLayer::HostileProj,
                GameLayer::Pickups,
            ],
        ),
    )
}

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            &mut MovementController,
            &mut PlayerAnimation,
            &mut Transform,
        ),
        With<Player>,
    >,
) {
    for (mut controller, mut animation, mut transform) in &mut query {
        let mut pressed_flag: bool = false;
        // Collect directional input.
        let mut intent = Vec2::ZERO;
        if input.pressed(KeyCode::KeyW) {
            intent.y += 1.0;
            animation.direction = PlayerDirection::Up;
            pressed_flag = true;
        }
        if input.pressed(KeyCode::KeyS) {
            intent.y -= 1.0;
            animation.direction = PlayerDirection::Down;
            pressed_flag = true;
        }
        if input.pressed(KeyCode::KeyA) {
            intent.x -= 1.0;
            animation.direction = PlayerDirection::Left;
            pressed_flag = true;
            transform.scale.x = -1.;
        }
        if input.pressed(KeyCode::KeyD) {
            intent.x += 1.0;
            animation.direction = PlayerDirection::Right;
            pressed_flag = true;
            transform.scale.x = 1.;
        }
        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        // This should be omitted if the input comes from an analog stick instead.
        let intent = intent.normalize_or_zero();
        controller.intent = intent;
        if pressed_flag {
            animation.state = PlayerAnimationState::Walk;
        } else {
            animation.state = PlayerAnimationState::Idle;
        }
    }
}

#[derive(Asset, Clone, Reflect)]
pub struct PlayerAssets {
    pub aseprite: Handle<Aseprite>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}
