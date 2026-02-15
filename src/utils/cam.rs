use bevy::{camera::*, prelude::*};
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{
    asset_tracking::ResourceHandles, game::player::Player, menus::Menu, screens::Screen,
    theme::widget,
};

pub const FOLLOW_CAMERA_TRESHOLD: f32 = 100.0; // Determine based on the character speed
pub const FOLLOW_CAMERA_MAX_SPEED: f32 = 1000.0;
pub const FOLLOW_CAMERA_BASE_SPEED: f32 = 4.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, update_camera);
}

fn spawn_camera(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::ZERO,
            right: Val::ZERO,
            width: percent(6.5),
            height: Val::Auto,
            margin: UiRect::all(Val::Px(5.)),
            ..default()
        },
        ImageNode::new(asset_server.load_with_settings(
            "images/powered-by-bevy.png",
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::linear();
            },
        )),
    ));
}

fn update_camera(
    player_query: Single<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    time: Res<Time>,
) {
    let player_transform = player_query;
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let camera_pos = camera_transform.translation.truncate();
        let player_pos = player_transform.translation.truncate();
        let d = camera_pos.distance(player_pos);

        // smoothing
        let factor = (d / FOLLOW_CAMERA_TRESHOLD)
            .clamp(1.0, FOLLOW_CAMERA_MAX_SPEED / FOLLOW_CAMERA_BASE_SPEED);
        let effective_speed = FOLLOW_CAMERA_BASE_SPEED * factor;

        let pos: Vec2 = camera_pos.lerp(player_pos, effective_speed * time.delta_secs());
        //camera_transform.translation.x = pos.x; // life hax so it dont go out of map
        camera_transform.translation.y = pos.y;
    }
}
