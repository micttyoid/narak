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
use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    game::{
        Red,
        animation::*,
        level::{
            bosses::{Boss, PHASE_1_NAME, PHASE_2_NAME, PHASE_3_NAME, TUTORIAL_BOSS_NAME},
            enemies::*,
            projectiles::*,
        },
        player::*,
    },
};
use avian2d::prelude::*;
use bevy::{ecs::system::SystemParam, input::common_conditions::input_just_pressed, prelude::*};
use rand::seq::IndexedRandom;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_player_movement,
            apply_screen_wrap,
            apply_player_throw.run_if(input_just_pressed(MouseButton::Left)),
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

#[derive(SystemParam)]
pub struct PassthroughHook<'w, 's> {
    projectile_query: Query<'w, 's, &'static Projectile>,
    passthrough_query: Query<'w, 's, &'static ProjectilePassthrough>,
    recalled_query: Query<'w, 's, &'static Recalled>,
    player_query: Query<'w, 's, &'static Player>,
}

impl CollisionHooks for PassthroughHook<'_, '_> {
    fn filter_pairs(&self, collider1: Entity, collider2: Entity, _commands: &mut Commands) -> bool {
        // Recalled Projectile
        let is_c1_recalled = self.recalled_query.contains(collider1);
        let is_c2_recalled = self.recalled_query.contains(collider2);

        if is_c1_recalled || is_c2_recalled {
            let is_c1_player = self.player_query.contains(collider1);
            let is_c2_player = self.player_query.contains(collider2);

            // If one entity is Recalled, the OTHER must be the player to collide.
            // Otherwise, ignore the collision entirely.
            if is_c1_recalled && !is_c2_player {
                return false;
            }
            if is_c2_recalled && !is_c1_player {
                return false;
            }
        }

        let is_projectile1 = self.projectile_query.get(collider1).is_ok();
        let is_projectile2 = self.projectile_query.get(collider2).is_ok();

        if is_projectile1 == is_projectile2 {
            return true;
        }

        let (_projectile, passthrough) = if is_projectile1 {
            (collider1, collider2)
        } else {
            (collider2, collider1)
        };
        self.passthrough_query.get(passthrough).is_err()
    }
}

/// Collision handling; runs in FixedUpdate. [`restore_ammo`] must run after this
/// so ammo is restored when friendly projectiles are despawned on hit.
#[cfg_attr(any(), rustfmt::skip)]
pub(crate) fn on_collision(
    mut commands: Commands,
    anim_assets: Res<AnimationAssets>,
    mut collision_reader: MessageReader<CollisionStart>,
    mut enemy_query: Query<(Entity, &mut Enemy, Option<&Boss>, Option<&Name>)>,
    mut player_query: Query<(Entity, &mut Player)>,
    mut projectile_query: Query<(
        Entity,
        &mut Projectile,
        &mut Transform,
        &mut LinearVelocity,
        Has<Friendly>,
        Has<Hostile>,
    )>,
    mut something_else_query: Query<&ProjectilePassthrough>,
) {
    for msg in collision_reader.read() {
        let c1 = msg.collider1;
        let c2 = msg.collider2;

        let mut is_c1_projectile: Option<bool> = None;
        let mut is_c2_projectile: Option<bool> = None;

        // player/enemy with projectile
        if on_collision_player(&mut commands, &anim_assets, &mut enemy_query, &mut player_query, &mut projectile_query, &mut something_else_query, &c1, &c2, &mut is_c2_projectile)
        || on_collision_player(&mut commands, &anim_assets, &mut enemy_query, &mut player_query, &mut projectile_query, &mut something_else_query, &c2, &c1, &mut is_c1_projectile)
        || on_collision_enemy(&mut commands, &anim_assets, &mut enemy_query, &mut player_query, &mut projectile_query,  &mut something_else_query, &c1, &c2, &mut is_c2_projectile)
        || on_collision_enemy(&mut commands, &anim_assets, &mut enemy_query, &mut player_query, &mut projectile_query,  &mut something_else_query, &c2, &c1, &mut is_c1_projectile)
        {
            continue;
        }

        match (
            is_c1_projectile.unwrap_or(projectile_query.contains(c1)),
            is_c2_projectile.unwrap_or(projectile_query.contains(c2)),
        ) {
            (true, true) => {commands.spawn(sound_effect(anim_assets.projectiles.ricochet.choose(&mut rand::rng()).unwrap().clone()));},
            (true, false) => on_collision_projectile_with_something_else(&mut commands, &anim_assets, &mut projectile_query, &mut something_else_query, &c1, &c2),
            (false, true) =>  on_collision_projectile_with_something_else(&mut commands, &anim_assets, &mut projectile_query, &mut something_else_query, &c2, &c1),
            (false, false) => {/* else vs else */}
        }
    }
}

// return is_continue
fn on_collision_player(
    commands: &mut Commands,
    anim_assets: &Res<AnimationAssets>,
    enemy_query: &mut Query<(Entity, &mut Enemy, Option<&Boss>, Option<&Name>)>,
    player_query: &mut Query<(Entity, &mut Player)>,
    projectile_query: &mut Query<(
        Entity,
        &mut Projectile,
        &mut Transform,
        &mut LinearVelocity,
        Has<Friendly>,
        Has<Hostile>,
    )>,
    something_else_query: &mut Query<&ProjectilePassthrough>,
    c1: &Entity,
    c2: &Entity,
    is_c2_projectile: &mut Option<bool>,
) -> bool {
    // c1 is player and c2 is projectile
    if let Ok((player_entity, mut player)) = player_query.get_mut(*c1) {
        if let Ok((proj_entity, _, _, _, _, has_hostile)) = projectile_query.get(*c2) {
            if has_hostile {
                commands.entity(player_entity).insert(Red::default());
                player.life = player.life.saturating_sub(1);
                commands.spawn(sound_effect(
                    anim_assets
                        .player
                        .damages
                        .choose(&mut rand::rng())
                        .unwrap()
                        .clone(),
                ));
            }
            commands.entity(proj_entity).despawn();
            *is_c2_projectile = Some(true);
        } else {
            *is_c2_projectile = Some(false);
        }
        return true;
    }
    false
}

// return is_continue
fn on_collision_enemy(
    commands: &mut Commands,
    anim_assets: &Res<AnimationAssets>,
    enemy_query: &mut Query<(Entity, &mut Enemy, Option<&Boss>, Option<&Name>)>,
    player_query: &mut Query<(Entity, &mut Player)>,
    projectile_query: &mut Query<(
        Entity,
        &mut Projectile,
        &mut Transform,
        &mut LinearVelocity,
        Has<Friendly>,
        Has<Hostile>,
    )>,
    something_else_query: &mut Query<&ProjectilePassthrough>,
    c1: &Entity,
    c2: &Entity,
    is_c2_projectile: &mut Option<bool>,
) -> bool {
    // c1 is enemy and c2 is projectile
    if let Ok((enemy_entity, mut enemy, opt_boss, opt_name)) = enemy_query.get_mut(*c1) {
        if let Ok((proj_entity, _, _, _, has_friendly, _)) = projectile_query.get(*c2) {
            if has_friendly {
                // Enemy got hit!
                enemy.life = enemy.life.saturating_sub(1);
                commands.entity(proj_entity).despawn();
                commands.entity(enemy_entity).insert(Red::default());
                if let (Some(_boss), Some(name)) = (opt_boss, opt_name) {
                    match name.as_str() {
                        TUTORIAL_BOSS_NAME => commands.spawn(sound_effect(
                            anim_assets
                                .enemies
                                .eye_enemy
                                .damages
                                .choose(&mut rand::rng())
                                .unwrap()
                                .clone(),
                        )),
                        PHASE_1_NAME => commands.spawn(sound_effect(
                            anim_assets
                                .enemies
                                .phase1
                                .damages
                                .choose(&mut rand::rng())
                                .unwrap()
                                .clone(),
                        )),
                        PHASE_2_NAME => commands.spawn(sound_effect(
                            anim_assets
                                .enemies
                                .phase2
                                .damages
                                .choose(&mut rand::rng())
                                .unwrap()
                                .clone(),
                        )),
                        PHASE_3_NAME => commands.spawn(sound_effect(
                            anim_assets
                                .enemies
                                .phase3
                                .damages
                                .choose(&mut rand::rng())
                                .unwrap()
                                .clone(),
                        )),

                        _ => {
                            panic!("unknown boss")
                        }
                    };
                } else {
                    // non-boss
                    commands.spawn(sound_effect(
                        anim_assets
                            .enemies
                            .eye_enemy
                            .damages
                            .choose(&mut rand::rng())
                            .unwrap()
                            .clone(),
                    ));
                }
            }
            // nothing for enemy bullet to enemy(drain)
            *is_c2_projectile = Some(true);
        } else {
            *is_c2_projectile = Some(false);
        }
        return true;
    }
    false
    // NOTE: nothing for enemy-to-enemy collision
}

fn on_collision_projectile_with_something_else(
    commands: &mut Commands,
    anim_assets: &Res<AnimationAssets>,
    projectile_query: &mut Query<(
        Entity,
        &mut Projectile,
        &mut Transform,
        &mut LinearVelocity,
        Has<Friendly>,
        Has<Hostile>,
    )>,
    something_else_query: &mut Query<&ProjectilePassthrough>,
    c1: &Entity,
    c2: &Entity,
) {
    if let Ok((proj_entity, mut projectile, mut transform, mut velocity, has_friendly, _)) =
        projectile_query.get_mut(*c1)
    {
        if let Ok(projectile_passthrough) = something_else_query.get_mut(*c2) {
            // nothing (replaced with hook)
        }
        if has_friendly {
            for due in projectile.dues.iter_mut() {
                match due {
                    Due::BounceDown(count) => {
                        match count {
                            1 => {
                                // Last bounce: stop the chakram then remove and restore
                                velocity.0 = Vec2::ZERO;
                                // commands.entity(proj_entity).despawn();
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
        } else {
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
    player: Single<(Entity, &Transform, &mut Player), With<Cool>>,
    global_transform: Query<&GlobalTransform>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    projectiles: Query<Entity, (With<Projectile>, With<Friendly>, Without<Recalled>)>,
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
            PLAYER_COLLIDER_CAPSULE.0,
            PLAYER_COLLIDER_CAPSULE.1,
            &anim_assets,
        ));
        commands.spawn(sound_effect(
            anim_assets
                .player
                .attacks
                .choose(&mut rand::rng())
                .unwrap()
                .clone(),
        ));
        player.decrement_ammo(1);

        // update cool
        commands.entity(player_entity).remove::<Cool>();
        commands.spawn(Cool::new(player.cool));
    } else {
        for entity in &projectiles {
            commands.entity(entity).insert(Recalled);
        }
    }
    // if ammo is out nah.
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(mut wrap_query: Query<&mut Transform, With<ScreenWrap>>) {
    let map_size = Vec2::new(640.0, 384.0);
    let half_size = map_size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(map_size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}
