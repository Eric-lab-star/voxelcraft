//! A day/night cycle: the sun orbits, its colour/brightness shifts, and the sky
//! and ambient light follow — day → sunset → night → sunrise.

use bevy::prelude::*;
use std::f32::consts::TAU;

/// Seconds for one full day/night cycle. Unused while the cycle is frozen at
/// noon, but kept for when the day/night loop is switched back on.
#[allow(dead_code)]
const DAY_LENGTH: f32 = 600.0;

/// Normalised time of day in `0.0..1.0` (0 = sunrise-ish).
#[derive(Resource)]
pub struct GameClock {
    pub t: f32,
}

impl Default for GameClock {
    fn default() -> Self {
        // Locked at noon for now — full daylight so the held block stays visible.
        GameClock { t: 0.25 }
    }
}

/// Marks the directional light that acts as the sun.
#[derive(Component)]
pub struct Sun;

pub fn day_night(
    clock: Res<GameClock>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut fog: Query<&mut DistanceFog>,
) {
    // Day/night cycle is frozen for now — the clock stays put (see `GameClock`
    // default), so the sun holds at noon and the world never darkens.
    let angle = clock.t * TAU;
    let elevation = angle.sin(); // -1 (midnight) .. 1 (noon)

    if let Ok((mut transform, mut light)) = sun.single_mut() {
        // Sun orbits overhead, slightly tilted so shadows have direction.
        let sun_pos = Vec3::new(angle.cos() * 120.0, elevation * 120.0 + 6.0, 40.0);
        *transform = Transform::from_translation(sun_pos).looking_at(Vec3::ZERO, Vec3::Y);

        let day = elevation.max(0.0);
        // Faint moonlight floor so nights aren't pitch black.
        light.illuminance = 300.0 + day.powf(0.5) * 11_000.0;
        // Warm the light near the horizon (sunrise/sunset).
        let warm = (1.0 - elevation.abs() * 3.0).clamp(0.0, 1.0);
        light.color = Color::srgb(1.0, 0.97 - warm * 0.22, 0.92 - warm * 0.45);
    }

    // Sky colour: night → day base, tinted orange near the horizon.
    let daylight = smoothstep(-0.08, 0.28, elevation);
    let horizon = (1.0 - elevation.abs() * 4.0).clamp(0.0, 1.0);
    let night_sky = Vec3::new(0.02, 0.03, 0.09);
    let day_sky = Vec3::new(0.53, 0.74, 0.92);
    let sunset = Vec3::new(0.86, 0.46, 0.28);
    let sky = night_sky.lerp(day_sky, daylight).lerp(sunset, horizon * 0.55);
    let sky_color = Color::srgb(sky.x, sky.y, sky.z);
    clear.0 = sky_color;
    // Fade distant terrain into whatever colour the sky currently is.
    if let Ok(mut fog) = fog.single_mut() {
        fog.color = sky_color;
    }

    // Ambient light dims at night.
    ambient.brightness = 20.0 + daylight * 155.0;
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
