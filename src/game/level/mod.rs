use avian2d::prelude::{Physics, PhysicsTime};
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut time: ResMut<Time<Physics>>,
) {
    time.unpause();
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            player(100.0, &player_assets),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )
        ],
    ));
}

// A simplistic controller
/*
pub const MOVE_SPEED: f32 = 200.;
fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player: Query<&mut LinearVelocity, With<PlayerMarker>>,
) {
    for mut rb_vel in player.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= Vec2::new(1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction += Vec2::new(0.0, 1.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction -= Vec2::new(0.0, 1.0);
        }

        if direction != Vec2::ZERO {
            direction /= direction.length();
        }

        rb_vel.0 = direction * MOVE_SPEED;
    }
}
*/
/*
#[derive(Default)]
struct HelperPlugin;

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            //EguiPlugin::default(),
            //WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        ));
         app.add_systems(Update, camera_movement);
        app.add_systems(Update, map_rotate);
    }
}

const MINIMUM_SCALE: f32 = 0.1;

// A simple camera system for moving and zooming the camera.
fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera>>,
) {
    for (mut transform, mut projection) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let Projection::Orthographic(ref mut ortho) = *projection else {
            continue;
        };

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::KeyZ) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::KeyX) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < MINIMUM_SCALE {
            ortho.scale = MINIMUM_SCALE;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_secs() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

const ROTATION_SPEED: f32 = 45.;

#[allow(clippy::type_complexity)]
fn map_rotate(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut world_or_map_query: Query<
        (Option<&ChildOf>, Option<&TiledMap>, &mut Transform),
        Or<(With<TiledMap>, With<TiledWorld>)>,
    >,
) {
    for (parent, map_marker, mut transform) in world_or_map_query.iter_mut() {
        // If we have a map with a parent entity, it probably means this map belongs to a world
        // and we should rotate the world instead of the map
        if parent.is_some() && map_marker.is_some() {
            continue;
        }
        if keyboard_input.pressed(KeyCode::KeyQ) {
            transform.rotate_z(f32::to_radians(ROTATION_SPEED * time.delta_secs()));
        }

        if keyboard_input.pressed(KeyCode::KeyE) {
            transform.rotate_z(f32::to_radians(-(ROTATION_SPEED * time.delta_secs())));
        }
    }
}
*/
