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
        level::enemies::{
            EnemyAssets, EyeEnemyAssets, GatesAssets, MayaAssets, MuraAssets, NarakAssets,
        },
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
                ase_sprite_animation.animation.play_loop("Idle");
            }
            PlayerAnimationState::Walk => {
                ase_sprite_animation.animation.play_loop("Walk");
            }
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
}

#[derive(Reflect, PartialEq, Default)]
pub enum PlayerAnimationState {
    Walk,
    #[default]
    Idle,
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
                aseprite: assets.load("textures/chars/mom.aseprite"),
                steps: vec![
                    assets.load("audio/sound_effects/player/step1.ogg"),
                    assets.load("audio/sound_effects/player/step2.ogg"),
                    assets.load("audio/sound_effects/player/step3.ogg"),
                    assets.load("audio/sound_effects/player/step4.ogg"),
                ],
                attacks: vec![
                    assets.load("audio/sound_effects/player/att1.ogg"),
                    assets.load("audio/sound_effects/player/att2.ogg"),
                    assets.load("audio/sound_effects/player/att3.ogg"),
                ],
                pickup: assets.load("textures/props/pickup.ogg"),
                damages: vec![
                    assets.load("audio/sound_effects/player/dmg1.ogg"),
                    assets.load("audio/sound_effects/player/dmg2.ogg"),
                    assets.load("audio/sound_effects/player/dmg3.ogg"),
                    assets.load("audio/sound_effects/player/dmg4.ogg"),
                ],
                chakram: assets.load("textures/props/chakram.aseprite"),
            },
            enemies: EnemyAssets {
                seedlng_aseprite: assets.load("textures/chars/seedling.aseprite"),
                eye_enemy: EyeEnemyAssets {
                    aseprite: assets.load("textures/chars/eye-enemy.aseprite"),
                    damages: vec![
                        assets.load("audio/sound_effects/enemies/eyes/dmg1.ogg"),
                        assets.load("audio/sound_effects/enemies/eyes/dmg2.ogg"),
                        assets.load("audio/sound_effects/enemies/eyes/dmg3.ogg"),
                        assets.load("audio/sound_effects/enemies/eyes/dmg4.ogg"),
                    ],
                },
                // boss1
                gates: GatesAssets {
                    aseprite: assets.load("textures/chars/boss1.aseprite"),
                    attacks: vec![],
                    damages: vec![],
                },
                // boss2
                maya: MayaAssets {
                    aseprite: assets.load("textures/chars/boss2.aseprite"),
                    attacks: vec![
                        assets.load("audio/sound_effects/maya/att1.ogg"),
                        assets.load("audio/sound_effects/maya/att2.ogg"),
                        assets.load("audio/sound_effects/maya/att3.ogg"),
                        assets.load("audio/sound_effects/maya/att4.ogg"),
                    ],
                    damages: vec![
                        assets.load("audio/sound_effects/maya/dmg1.ogg"),
                        assets.load("audio/sound_effects/maya/dmg2.ogg"),
                        assets.load("audio/sound_effects/maya/dmg3.ogg"),
                        assets.load("audio/sound_effects/maya/dmg4.ogg"),
                    ],
                    grunts: vec![
                        assets.load("audio/sound_effects/maya/grunt1.ogg"),
                        assets.load("audio/sound_effects/maya/grunt2.ogg"),
                        assets.load("audio/sound_effects/maya/grunt3.ogg"),
                    ],
                    intro: assets.load("audio/sound_effects/maya/intro.ogg"),
                },
                // boss3
                mura: MuraAssets {
                    aseprite: assets.load("textures/chars/boss3.aseprite"),
                    attacks: vec![
                        assets.load("audio/sound_effects/mura/att1.ogg"),
                        assets.load("audio/sound_effects/mura/att2.ogg"),
                    ],
                    damages: vec![
                        assets.load("audio/sound_effects/mura/dmg1.ogg"),
                        assets.load("audio/sound_effects/mura/dmg2.ogg"),
                        assets.load("audio/sound_effects/mura/dmg3.ogg"),
                    ],
                    idle: assets.load("audio/sound_effects/mura/idle.ogg"),
                    enemy: assets.load("textures/chars/snake.aseprite"),
                },
                // boss4
                narak: NarakAssets {
                    aseprite: assets.load("textures/chars/boss4.aseprite"),
                    attacks: vec![
                        assets.load("audio/sound_effects/narak/att1.ogg"),
                        assets.load("audio/sound_effects/narak/att2.ogg"),
                    ],
                    damages: vec![
                        assets.load("audio/sound_effects/narak/dmg1.ogg"),
                        assets.load("audio/sound_effects/narak/dmg2.ogg"),
                        assets.load("audio/sound_effects/narak/dmg3.ogg"),
                    ],
                    death: assets.load("audio/sound_effects/narak/death.ogg"),
                    enemy: assets.load("textures/chars/ashiok.aseprite"),
                },
                bullet: assets.load("textures/props/bullet.png"),
            },
        }
    }
}
