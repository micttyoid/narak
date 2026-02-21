use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::math::ops::powf;
use bevy::{camera::*, prelude::*};

use crate::game::player::Player;

pub const FOLLOW_CAMERA_TRESHOLD: f32 = 100.0;
pub const FOLLOW_CAMERA_MAX_SPEED: f32 = 1000.0;
pub const FOLLOW_CAMERA_BASE_SPEED: f32 = 4.5;

// Shake Constants
const TRAUMA_DECAY_PER_SECOND: f32 = 1.0; // Decays fully in 1 second
const TRAUMA_EXPONENT: f32 = 2.0;
const MAX_ANGLE: f32 = 5.0_f32; // Converted to radians below
const MAX_TRANSLATION: f32 = 15.0;
const NOISE_SPEED: f32 = 20.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    // Reset transform before game logic
    app.add_systems(PreUpdate, reset_transform);
    // Normal follow camera logic
    app.add_systems(Update, update_camera);
    // Apply shake right before rendering
    app.add_systems(PostUpdate, shake_camera.before(TransformSystems::Propagate));
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
        CameraShakeConfig {
            trauma_decay_per_second: TRAUMA_DECAY_PER_SECOND,
            exponent: TRAUMA_EXPONENT,
            max_angle: MAX_ANGLE.to_radians(),
            max_translation: MAX_TRANSLATION,
            noise_speed: NOISE_SPEED,
        },
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
        let mut pos: Vec2 = camera_pos.lerp(player_pos, effective_speed * time.delta_secs());
        //camera_transform.translation.x = pos.x; // life hax so it dont go out of map
        pos.y = pos.y.clamp(-300.0, 300.0);
        camera_transform.translation.y = pos.y;
    }
}

#[derive(Component, Debug, Default)]
pub struct CameraShakeState {
    pub trauma: f32,
    original_transform: Transform,
}

#[derive(Component, Debug)]
#[require(CameraShakeState)]
pub struct CameraShakeConfig {
    pub trauma_decay_per_second: f32,
    pub exponent: f32,
    pub max_angle: f32,
    pub max_translation: f32,
    pub noise_speed: f32,
}

fn reset_transform(mut camera_shake_q: Query<(&CameraShakeState, &mut Transform)>) {
    for (camera_shake, mut transform) in &mut camera_shake_q {
        *transform = camera_shake.original_transform;
    }
}

fn shake_camera(
    mut camera_shake_q: Query<(&mut CameraShakeState, &CameraShakeConfig, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut camera_shake, config, mut transform) in &mut camera_shake_q {
        camera_shake.original_transform = *transform;

        if camera_shake.trauma <= 0.0 {
            continue;
        }

        let t = time.elapsed_secs() * config.noise_speed;
        let rotation_noise = perlin_noise::generate(t);
        let x_noise = perlin_noise::generate(t + 100.0);
        let y_noise = perlin_noise::generate(t + 200.0);

        let shake = powf(camera_shake.trauma, config.exponent);

        let roll_offset = rotation_noise * shake * config.max_angle;
        let x_offset = x_noise * shake * config.max_translation;
        let y_offset = y_noise * shake * config.max_translation;

        transform.translation.x += x_offset;
        transform.translation.y += y_offset;
        transform.rotate_z(roll_offset);

        camera_shake.trauma -= config.trauma_decay_per_second * time.delta_secs();
        camera_shake.trauma = camera_shake.trauma.clamp(0.0, 1.0);
    }
}

mod perlin_noise {
    pub fn generate(x: f32) -> f32 {
        let x_floor = x.floor() as usize;
        let xf0 = x - x_floor as f32;
        let xf1 = xf0 - 1.0;
        let xi0 = x_floor & 0xFF;
        let xi1 = (x_floor + 1) & 0xFF;
        let t = fade(xf0).clamp(0.0, 1.0);
        let h0 = PERMUTATION_TABLE[xi0];
        let h1 = PERMUTATION_TABLE[xi1];
        let a = dot_grad(h0, xf0);
        let b = dot_grad(h1, xf1);
        a + t * (b - a)
    }

    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn dot_grad(hash: u8, xf: f32) -> f32 {
        if hash & 0x1 != 0 { xf } else { -xf }
    }

    const PERMUTATION_TABLE: [u8; 256] = [
        0x97, 0xA0, 0x89, 0x5B, 0x5A, 0x0F, 0x83, 0x0D, 0xC9, 0x5F, 0x60, 0x35, 0xC2, 0xE9, 0x07,
        0xE1, 0x8C, 0x24, 0x67, 0x1E, 0x45, 0x8E, 0x08, 0x63, 0x25, 0xF0, 0x15, 0x0A, 0x17, 0xBE,
        0x06, 0x94, 0xF7, 0x78, 0xEA, 0x4B, 0x00, 0x1A, 0xC5, 0x3E, 0x5E, 0xFC, 0xDB, 0xCB, 0x75,
        0x23, 0x0B, 0x20, 0x39, 0xB1, 0x21, 0x58, 0xED, 0x95, 0x38, 0x57, 0xAE, 0x14, 0x7D, 0x88,
        0xAB, 0xA8, 0x44, 0xAF, 0x4A, 0xA5, 0x47, 0x86, 0x8B, 0x30, 0x1B, 0xA6, 0x4D, 0x92, 0x9E,
        0xE7, 0x53, 0x6F, 0xE5, 0x7A, 0x3C, 0xD3, 0x85, 0xE6, 0xDC, 0x69, 0x5C, 0x29, 0x37, 0x2E,
        0xF5, 0x28, 0xF4, 0x66, 0x8F, 0x36, 0x41, 0x19, 0x3F, 0xA1, 0x01, 0xD8, 0x50, 0x49, 0xD1,
        0x4C, 0x84, 0xBB, 0xD0, 0x59, 0x12, 0xA9, 0xC8, 0xC4, 0x87, 0x82, 0x74, 0xBC, 0x9F, 0x56,
        0xA4, 0x64, 0x6D, 0xC6, 0xAD, 0xBA, 0x03, 0x40, 0x34, 0xD9, 0xE2, 0xFA, 0x7C, 0x7B, 0x05,
        0xCA, 0x26, 0x93, 0x76, 0x7E, 0xFF, 0x52, 0x55, 0xD4, 0xCF, 0xCE, 0x3B, 0xE3, 0x2F, 0x10,
        0x3A, 0x11, 0xB6, 0xBD, 0x1C, 0x2A, 0xDF, 0xB7, 0xAA, 0xD5, 0x77, 0xF8, 0x98, 0x02, 0x2C,
        0x9A, 0xA3, 0x46, 0xDD, 0x99, 0x65, 0x9B, 0xA7, 0x2B, 0xAC, 0x09, 0x81, 0x16, 0x27, 0xFD,
        0x13, 0x62, 0x6C, 0x6E, 0x4F, 0x71, 0xE0, 0xE8, 0xB2, 0xB9, 0x70, 0x68, 0xDA, 0xF6, 0x61,
        0xE4, 0xFB, 0x22, 0xF2, 0xC1, 0xEE, 0xD2, 0x90, 0x0C, 0xBF, 0xB3, 0xA2, 0xF1, 0x51, 0x33,
        0x91, 0xEB, 0xF9, 0x0E, 0xEF, 0x6B, 0x31, 0xC0, 0xD6, 0x1F, 0xB5, 0xC7, 0x6A, 0x9D, 0xB8,
        0x54, 0xCC, 0xB0, 0x73, 0x79, 0x32, 0x2D, 0x7F, 0x04, 0x96, 0xFE, 0x8A, 0xEC, 0xCD, 0x5D,
        0xDE, 0x72, 0x43, 0x1D, 0x18, 0x48, 0xF3, 0x8D, 0x80, 0xC3, 0x4E, 0x42, 0xD7, 0x3D, 0x9C,
        0xB4,
    ];
}
