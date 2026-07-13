//! A first-person walking player: mouse look, WASD movement, gravity, jumping,
//! and axis-by-axis AABB collision against the voxel world.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::world::{World, PLAY_MARGIN, WORLD_X, WORLD_Z};

/// Half-extents of the player's collision box (0.6 × 1.8 × 0.6 blocks).
const HALF: Vec3 = Vec3::new(0.3, 0.9, 0.3);
/// Eye height above the box centre (so the camera sits near the top).
const EYE_OFFSET: f32 = 0.7;
const WALK_SPEED: f32 = 4.8;
const SPRINT_SPEED: f32 = 7.5;
// Stronger gravity + matching jump impulse => snappier, less floaty jumps that
// still clear a 1-block step (peak height ≈ JUMP_SPEED² / 2·GRAVITY ≈ 1.25).
const GRAVITY: f32 = 34.0;
const JUMP_SPEED: f32 = 9.2;
const TERMINAL: f32 = 55.0;
/// Nudge used when snapping out of a block so we don't re-collide next frame.
const EPS: f32 = 1.0e-3;

#[derive(Component)]
pub struct Player {
    /// Centre of the collision box in world space.
    pub center: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl Player {
    pub fn new(center: Vec3) -> Self {
        Player {
            center,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: -0.2,
            on_ground: false,
        }
    }

    /// The eye (camera) position: near the top of the collision box.
    pub fn eye(&self) -> Vec3 {
        self.center + Vec3::new(0.0, EYE_OFFSET, 0.0)
    }

    /// The look rotation built from yaw (around Y) then pitch (around X).
    pub fn look_rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch)
    }

    /// Unit vector the player is looking along.
    pub fn forward(&self) -> Vec3 {
        self.look_rotation() * Vec3::NEG_Z
    }

    /// Does the player's collision box overlap the unit cell at `block`?
    /// Used to stop the player placing a block inside themselves.
    pub fn intersects_block(&self, block: IVec3) -> bool {
        let min = self.center - HALF;
        let max = self.center + HALF;
        let bmin = block.as_vec3();
        let bmax = bmin + Vec3::ONE;
        min.x < bmax.x
            && max.x > bmin.x
            && min.y < bmax.y
            && max.y > bmin.y
            && min.z < bmax.z
            && max.z > bmin.z
    }
}

// --- Cursor grab (shared UX from the fly-camera version) -------------------

pub fn grab_cursor(mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut cursor) = cursors.single_mut() {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

pub fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if let Ok(mut cursor) = cursors.single_mut() {
        match cursor.grab_mode {
            CursorGrabMode::Locked => {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
            _ => {
                cursor.grab_mode = CursorGrabMode::Locked;
                cursor.visible = false;
            }
        }
    }
}

// --- Look ------------------------------------------------------------------

pub fn player_look(
    mouse: Res<AccumulatedMouseMotion>,
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    mut query: Query<&mut Player>,
    mut warmup: Local<u32>,
) {
    let grabbed = cursors
        .single()
        .map(|c| c.grab_mode != CursorGrabMode::None)
        .unwrap_or(false);
    if !grabbed {
        return;
    }
    let delta = mouse.delta;
    if delta == Vec2::ZERO {
        return;
    }
    // Ignore the large spurious delta produced when the cursor is first grabbed
    // and recentred (which can happen a few frames in, once the window gains
    // focus), so the view doesn't spin off on startup. A brief warm-up also
    // swallows the very first grabbed frames.
    if *warmup < 3 || delta.length() > 120.0 {
        *warmup += 1;
        return;
    }
    let sensitivity = 0.0025;
    for mut player in &mut query {
        player.yaw -= delta.x * sensitivity;
        player.pitch = (player.pitch - delta.y * sensitivity).clamp(-1.54, 1.54);
    }
}

// --- Movement + physics ----------------------------------------------------

pub fn player_physics(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    world: Res<World>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    // Clamp dt so a frame hitch can't launch the player through the ground.
    let dt = time.delta_secs().min(0.05);

    for (mut player, mut transform) in &mut query {
        // Horizontal facing (yaw only — looking up/down shouldn't change speed).
        let (sy, cy) = player.yaw.sin_cos();
        let forward = Vec3::new(-sy, 0.0, -cy);
        let right = Vec3::new(cy, 0.0, -sy);

        let mut wish = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            wish += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            wish -= forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            wish += right;
        }
        if keys.pressed(KeyCode::KeyA) {
            wish -= right;
        }
        wish.y = 0.0;
        if wish != Vec3::ZERO {
            wish = wish.normalize();
        }
        let speed = if keys.pressed(KeyCode::ControlLeft) {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        // Gravity + jump. Holding Space keeps hopping each time we land.
        player.velocity.y = (player.velocity.y - GRAVITY * dt).max(-TERMINAL);
        if player.on_ground && keys.pressed(KeyCode::Space) {
            player.velocity.y = JUMP_SPEED;
        }

        let delta = Vec3::new(
            wish.x * speed * dt,
            player.velocity.y * dt,
            wish.z * speed * dt,
        );

        // Resolve one axis at a time so we slide along walls instead of sticking.
        let mut center = player.center;
        step_axis(&world, &mut center, 0, delta.x);
        step_axis(&world, &mut center, 2, delta.z);

        player.on_ground = false;
        if step_axis(&world, &mut center, 1, delta.y) {
            if delta.y < 0.0 {
                player.on_ground = true;
            }
            player.velocity.y = 0.0;
        }
        player.center = center;

        // Camera = eye, positioned near the top of the box.
        transform.translation = center + Vec3::new(0.0, EYE_OFFSET, 0.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Y, player.yaw)
            * Quat::from_axis_angle(Vec3::X, player.pitch);
    }
}

/// Move the box centre along one axis by `amount`, then push it back out of any
/// solid block it entered. Returns whether a collision was resolved.
fn step_axis(world: &World, center: &mut Vec3, axis: usize, amount: f32) -> bool {
    if amount == 0.0 {
        return false;
    }
    axis_add(center, axis, amount);
    if !box_hits(world, *center, HALF) {
        return false;
    }

    if amount > 0.0 {
        let leading = axis_get(*center, axis) + HALF[axis];
        axis_set(center, axis, leading.floor() - HALF[axis] - EPS);
    } else {
        let trailing = axis_get(*center, axis) - HALF[axis];
        axis_set(center, axis, trailing.ceil() + HALF[axis] + EPS);
    }
    true
}

/// Does the AABB (centre ± half) overlap any movement-blocking voxel?
fn box_hits(world: &World, center: Vec3, half: Vec3) -> bool {
    let min = center - half;
    let max = center + half;
    let x0 = min.x.floor() as i32;
    let x1 = (max.x - EPS).floor() as i32;
    let y0 = min.y.floor() as i32;
    let y1 = (max.y - EPS).floor() as i32;
    let z0 = min.z.floor() as i32;
    let z1 = (max.z - EPS).floor() as i32;

    for y in y0..=y1 {
        for z in z0..=z1 {
            for x in x0..=x1 {
                // Invisible walls set a margin in from the world edge, so the
                // player is stopped while distant terrain stays visible.
                if x < PLAY_MARGIN
                    || x >= WORLD_X - PLAY_MARGIN
                    || z < PLAY_MARGIN
                    || z >= WORLD_Z - PLAY_MARGIN
                {
                    return true;
                }
                if world.get(x, y, z).blocks_movement() {
                    return true;
                }
            }
        }
    }
    false
}

#[inline]
fn axis_get(v: Vec3, a: usize) -> f32 {
    match a {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}

#[inline]
fn axis_set(v: &mut Vec3, a: usize, val: f32) {
    match a {
        0 => v.x = val,
        1 => v.y = val,
        _ => v.z = val,
    }
}

#[inline]
fn axis_add(v: &mut Vec3, a: usize, d: f32) {
    match a {
        0 => v.x += d,
        1 => v.y += d,
        _ => v.z += d,
    }
}
