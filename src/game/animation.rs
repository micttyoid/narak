//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use bevy_aseprite_ultra::{AsepriteUltraPlugin, prelude::*};
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    game::{
        level::enemies::EnemyAssets,
        player::{Player, PlayerAssets},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(AsepriteUltraPlugin);
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (update_animation_state, trigger_step_sound_effect)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// Update the sprite direction and animation state (idling/walking).
fn update_animation_state(mut anim_q: Query<(&mut AseAnimation, &PlayerAnimation), With<Player>>) {
    for (mut ase_sprite_animation, player) in anim_q.iter_mut() {
        match player.state {
            PlayerAnimationState::Idle => {
                ase_sprite_animation.animation.play_loop("idle");
            }
            PlayerAnimationState::Walk => match player.direction {
                PlayerDirection::Up => {
                    ase_sprite_animation.animation.play_loop("walk-up");
                }
                PlayerDirection::Down => {
                    ase_sprite_animation.animation.play_loop("walk-down");
                }
                PlayerDirection::Left | PlayerDirection::Right => {
                    ase_sprite_animation.animation.play_loop("walk-right");
                }
            },
        }
    }
}

/// If the player is moving, play a step sound effect synchronized with the
/// animation.
fn trigger_step_sound_effect(
    mut cmd: Commands,
    anim_assets: If<Res<AnimationAssets>>,
    mut anim_q: Query<&PlayerAnimation>,
    mut anim_msg: MessageReader<AnimationEvents>,
) {
    for msg in anim_msg.read() {
        for animation in &mut anim_q {
            if animation.state == PlayerAnimationState::Walk {
                match msg {
                    AnimationEvents::LoopCycleFinished(_entity) => {
                        let rng = &mut rand::rng();
                        let random_step = anim_assets.player.steps.choose(rng).unwrap().clone();
                        cmd.spawn(sound_effect(random_step));
                    }
                    AnimationEvents::Finished(_entity) => (),
                }
            }
        }
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to aseprite animation we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAnimation {
    pub state: PlayerAnimationState,
    pub direction: PlayerDirection,
}

#[derive(Reflect, PartialEq, Default)]
pub enum PlayerAnimationState {
    Walk,
    #[default]
    Idle,
}

#[derive(Reflect, PartialEq, Default)]
pub enum PlayerDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct AnimationAssets {
    pub player: PlayerAssets,
    pub enemies: EnemyAssets,
}

impl FromWorld for AnimationAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player: PlayerAssets {
                aseprite: assets.load("textures/chars/player.aseprite"),
                steps: vec![
                    assets.load("audio/sound_effects/step1.ogg"),
                    assets.load("audio/sound_effects/step2.ogg"),
                    assets.load("audio/sound_effects/step3.ogg"),
                    assets.load("audio/sound_effects/step4.ogg"),
                ],
            },
            enemies: EnemyAssets {
                aseprite: assets.load("textures/chars/seedling.aseprite"),
            },
        }
    }
}
