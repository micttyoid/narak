//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).
use std::ops::DerefMut;

use crate::{
    AppSystems, PausableSystems,
    game::{
        animation::*,
        level::{enemies::*, projectiles::*},
        player::*,
    },
};
use avian2d::prelude::*;
use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_player_movement,
            apply_screen_wrap,
            apply_player_throw.run_if(input_just_pressed(KeyCode::Space)),
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(FixedUpdate, (on_collision).in_set(PausableSystems));
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            max_speed: 1.0,
        }
    }
}

fn on_collision(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    mut enemy_query: Query<(Entity, &mut Enemy)>,
    mut player_query: Query<&mut Player>,
    mut projectile_query: Query<(Entity, &mut Projectile, Has<Friendly>, Has<Hostile>)>,
) {
    for msg in collision_reader.read() {
        let c1 = msg.collider1;
        let c2 = msg.collider2;
        if let Ok((proj_entity, mut projectile, has_friendly, _)) = projectile_query.get_mut(c1) {
            if let Ok((_enemy_entity, mut enemy)) = enemy_query.get_mut(c2)
                && has_friendly
            {
                // Enemy got hit!
                enemy.life = enemy.life.saturating_sub(1);
                commands.entity(proj_entity).despawn();
            } else if !player_query.contains(c1) {
                // This part getting smelly
                // Something else! (neither enemy nor player)
                for due in projectile.dues.iter_mut() {
                    match due {
                        Due::BounceDown(count) => {
                            match count {
                                1 => {
                                    // This goes to zero: remove and restore
                                    commands.entity(proj_entity).despawn();
                                }
                                0 => {
                                    //panic!("Bounce Down was not set correctly");
                                    commands.entity(proj_entity).despawn(); // should not happen
                                }
                                _ => {
                                    *count = count.saturating_sub(1);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        } else if player_query.contains(c1)
            && let Ok((proj_entity, _, _, has_hostile)) = projectile_query.get(c2)
        {
            let mut player = player_query
                .single_mut()
                .expect("Player does not exist or more than one player");
            if has_hostile {
                player.life = player.life.saturating_sub(1);
            }
            commands.entity(proj_entity).despawn();
        }
    }
}

fn apply_player_movement(mut movement_query: Query<(&MovementController, &mut LinearVelocity)>) {
    for (controller, mut rb_vel) in movement_query.iter_mut() {
        rb_vel.0 = controller.max_speed * controller.intent; // normal
    }
}

fn apply_player_throw(
    mut commands: Commands,
    anim_assets: Res<AnimationAssets>,
    mut player: Single<(Entity, &Transform, &mut Player), With<Cool>>,
    global_transform: Query<&GlobalTransform>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (player_entity, player_transform, mut player) = player.into_inner();

    if player.ammo != 0
        && let Ok(player_global_transform) = global_transform.get(player_entity)
    {
        let (x, y, _) = player_global_transform.translation().into(); // This may differ by the worldwrap
        let xy = Vec2::new(x, y);

        let (camera, camera_transform) = *camera_query;
        let dir_not_norm = if let Some(cursor_position) = window.cursor_position()
               // Calculate a world position based on the cursor's position.
               && let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            cursor_world_pos - xy
        } else {
            // Fallback
            player_transform.local_x().xy()
        };
        let direction = Dir2::new(dir_not_norm.normalize()).expect("It is not normalized");
        commands.spawn(player_chakra::<Friendly>(
            xy,
            direction,
            PLAYER_COLLIDER_RADIUS,
            &anim_assets,
        ));
        player.decrement_ammo(1);

        /*
        commands.spawn(bounce_down_projectile::<Friendly>(
            xy,
            direction,
            PLAYER_COLLIDER_RADIUS,
            &anim_assets,
        ));
        commands.spawn(lifespan_projectile::<Friendly>(
            xy,
            direction,
            PLAYER_COLLIDER_RADIUS,
            &anim_assets,
        ));
        */

        // update cool
        commands.entity(player_entity).remove::<Cool>();
        commands.spawn(Cool::new(player.cool));
    }
    // if ammo is out nah.
}

/// This should be where the optimization takes place if the frame dropss
fn apply_projectile_movement(
    mut movement_query: Query<(&Projectile, &mut LinearVelocity)>,
    mut commands: Commands,
    player: Single<(Entity, &Transform), With<Player>>,
) {
    let (player_entity, player_transform) = *player;

    let forward_direction = (*player_transform).forward();
    /*
    commands.entity(player_entity).add_child((
        Projectile,
    ));
    */
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
