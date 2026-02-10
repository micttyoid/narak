// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menus;
mod screens;
mod theme;
mod utils;

use bevy::{
    asset::{AssetMetaCheck, load_internal_binary_asset},
    prelude::*,
};

use avian2d::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::theme::palette::BACKGROUND_DARK;

//use bevy_ecs_tiled::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .build()
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy New 2D".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );

        // loading font as default - needs to be after default plugins are added
        load_internal_binary_asset!(
            app,
            TextFont::default().font,
            "../assets/fonts/samarn.ttf", // may change later or only use for certain places
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );

        app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
            .insert_resource({
                let mut physics_time = Time::<Physics>::default();
                physics_time.pause();
                physics_time
            });
        #[cfg(debug_assertions)]
        {
            app.add_plugins(PhysicsDebugPlugin);
        }
        app.add_plugins(TilemapPlugin);

        // Add other plugins.
        app.add_plugins((
            utils::plugin,
            asset_tracking::plugin,
            audio::plugin,
            game::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Background Color
        app.insert_resource(ClearColor(BACKGROUND_DARK));
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;
