//! A first-person walking player: mouse look, WASD movement, gravity, jumping,
//! and axis-by-axis AABB collision against the voxel world.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::block::Block;
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

// --- Water --------------------------------------------------------------
// Water is modelled as three forces on top of the usual gravity: buoyancy that
// scales with how much of the body is under the surface, a linear drag that
// bleeds off speed, and the swim strokes the player asks for.
//
// Buoyancy at full submersion is set equal to gravity, i.e. neutral: once you
// are completely under you hang at whatever depth you stopped at instead of
// bobbing back up, which is what makes the lakebed explorable. Break the
// surface and the submerged fraction — and with it the lift — drops, so you
// sink again unless you keep stroking upward.
const BUOYANCY: f32 = GRAVITY;
/// Exponential velocity damping per second at full submersion. Every terminal
/// speed below is `net_acceleration / WATER_DRAG`.
const WATER_DRAG: f32 = 3.4;
/// Upward acceleration from a swim stroke (Space). Sustained rise is this minus
/// the uncancelled gravity, so it settles the body ~40% submerged at the
/// surface — head and shoulders out — rather than rising forever.
const SWIM_UP_ACCEL: f32 = 20.0;
/// Downward acceleration when diving (Shift); ≈ 2.6 m/s sustained.
const SWIM_DOWN_ACCEL: f32 = 9.0;
/// Rise rate when hauling yourself up a wall from the water (see the climb-out
/// note in `player_physics`). Comfortably beats the 1-block bank you swam into.
const WATER_CLIMB_SPEED: f32 = 3.6;
/// Horizontal cruise speed while swimming, and its sprint variant.
const SWIM_SPEED: f32 = 3.1;
const SWIM_SPRINT_SPEED: f32 = 4.4;
/// How quickly horizontal velocity chases the input direction in water. Low
/// enough that starts and stops glide instead of snapping like they do on land.
const SWIM_ACCEL: f32 = 6.0;
/// Fastest you can move vertically in water, however you got there — this is
/// what turns a long fall into a splash-and-slow instead of a plunge.
const WATER_TERMINAL: f32 = 8.0;

// --- Flight (developer mode) -----------------------------------------------
// Toggled by double-tapping jump, the way Minecraft's creative flight is, so
// there is no extra key to learn. Flight still collides with the world: it is
// for getting around and building at height, not for passing through terrain.
const FLY_SPEED: f32 = 12.0;
const FLY_SPRINT_SPEED: f32 = 24.0;
/// How quickly flight velocity chases the input. Responsive, but enough lag that
/// you glide to a stop instead of halting dead in mid-air.
const FLY_ACCEL: f32 = 12.0;
/// Two jump presses within this many seconds toggle flight.
const FLY_TOGGLE_WINDOW: f32 = 0.35;

#[derive(Component)]
pub struct Player {
    /// Centre of the collision box in world space.
    pub center: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    /// Fraction of the collision box below the water surface, 0..1.
    pub submersion: f32,
    /// Whether the player is flying (developer mode).
    pub flying: bool,
}

impl Player {
    pub fn new(center: Vec3) -> Self {
        Player {
            center,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: -0.2,
            on_ground: false,
            submersion: 0.0,
            flying: false,
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
    mut toast: ResMut<crate::menu::Toast>,
    mut last_jump_tap: Local<f32>,
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
        let sprint = keys.pressed(KeyCode::ControlLeft);

        // Double-tap jump toggles flight. The tap time is tracked per system
        // rather than per player because there is only ever one of them.
        if keys.just_pressed(KeyCode::Space) {
            let now = time.elapsed_secs();
            if now - *last_jump_tap < FLY_TOGGLE_WINDOW {
                player.flying = !player.flying;
                player.velocity = Vec3::ZERO;
                *last_jump_tap = 0.0; // don't let a third tap re-toggle
                toast.show(if player.flying {
                    "비행 켜짐 — Space 상승, Shift 하강"
                } else {
                    "비행 꺼짐"
                });
            } else {
                *last_jump_tap = now;
            }
        }

        player.submersion = submersion(&world, player.center);
        let s = player.submersion;

        if player.flying {
            // --- Flying -----------------------------------------------------
            // No gravity and no buoyancy: velocity chases the input directly, so
            // you hover wherever you stop. Water is ignored entirely — being
            // dragged around by buoyancy while flying would be maddening.
            let speed = if sprint { FLY_SPRINT_SPEED } else { FLY_SPEED };
            let mut target = wish * speed;
            if keys.pressed(KeyCode::Space) {
                target.y += speed;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                target.y -= speed;
            }
            let k = 1.0 - (-FLY_ACCEL * dt).exp();
            let v = player.velocity;
            player.velocity = v + (target - v) * k;
        } else if s > 0.0 {
            // --- In water ---------------------------------------------------
            // Buoyancy scales with the submerged fraction, so the net vertical
            // force flips sign as you cross the surface and you settle there.
            player.velocity.y -= (GRAVITY - BUOYANCY * s) * dt;
            // Stroke thrust must NOT fade out with submersion the way buoyancy
            // does: you kick with your legs, at the bottom of the box, so the
            // thrust is there as long as they have water to push against. Fading
            // it linearly cancelled the stroke exactly as you tried to breach.
            let stroke = (s * 4.0).min(1.0);
            if keys.pressed(KeyCode::Space) {
                player.velocity.y += SWIM_UP_ACCEL * stroke * dt;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                player.velocity.y -= SWIM_DOWN_ACCEL * stroke * dt;
            }
            // Drag. Applied exponentially so it can't overshoot at any dt, and
            // scaled by submersion so wading is barely slowed.
            let damp = (-WATER_DRAG * s * dt).exp();
            player.velocity.y = (player.velocity.y * damp).clamp(-WATER_TERMINAL, WATER_TERMINAL);

            // Horizontal motion is momentum-based here (on land it is direct),
            // which is what gives swimming its glide.
            let target = wish * if sprint { SWIM_SPRINT_SPEED } else { SWIM_SPEED };
            let k = 1.0 - (-SWIM_ACCEL * dt).exp();
            player.velocity.x += (target.x - player.velocity.x) * k;
            player.velocity.z += (target.z - player.velocity.z) * k;
        } else {
            player.velocity.y = (player.velocity.y - GRAVITY * dt).max(-TERMINAL);
            let speed = if sprint { SPRINT_SPEED } else { WALK_SPEED };
            player.velocity.x = wish.x * speed;
            player.velocity.z = wish.z * speed;
        }

        // Jumping needs feet on something, not dry land: pushing off the lakebed
        // is how you surface from a shallow pool. Drag eats most of it when deep.
        // While flying, Space is the ascend control instead.
        if player.on_ground && !player.flying && keys.pressed(KeyCode::Space) {
            player.velocity.y = JUMP_SPEED;
        }

        // Resolve one axis at a time so we slide along walls instead of sticking.
        let mut center = player.center;
        // Zero the axis on impact so swim momentum doesn't pile up against a wall.
        let mut against_wall = false;
        if step_axis(&world, &mut center, 0, player.velocity.x * dt) {
            player.velocity.x = 0.0;
            against_wall = true;
        }
        if step_axis(&world, &mut center, 2, player.velocity.z * dt) {
            player.velocity.z = 0.0;
            against_wall = true;
        }

        // Pulling out onto a bank. No amount of swimming lifts you clear of the
        // water — buoyancy dies as you rise, which is realistic but leaves you
        // stranded, since the shore sits at the waterline and your feet never
        // reach it. So swimming into a wall while holding Space climbs it, the
        // way you'd haul yourself up on the edge. Requires an input direction, so
        // you don't creep up walls just by drifting into them.
        if !player.flying
            && s > 0.0
            && against_wall
            && wish != Vec3::ZERO
            && keys.pressed(KeyCode::Space)
        {
            player.velocity.y = player.velocity.y.max(WATER_CLIMB_SPEED);
        }

        player.on_ground = false;
        if step_axis(&world, &mut center, 1, player.velocity.y * dt) {
            if player.velocity.y < 0.0 {
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

/// What fraction of the player's box (0..1) sits inside water blocks.
///
/// Only the column under the box centre is sampled: water cells are full cubes
/// here regardless of their flow level, so a per-cell height test would be noise,
/// and a single column keeps the buoyancy from jittering as you drift over a
/// ragged shoreline.
fn submersion(world: &World, center: Vec3) -> f32 {
    let min_y = center.y - HALF.y;
    let max_y = center.y + HALF.y;
    let x = center.x.floor() as i32;
    let z = center.z.floor() as i32;

    let mut under = 0.0;
    for y in (min_y.floor() as i32)..=((max_y - EPS).floor() as i32) {
        if world.get(x, y, z) == Block::Water {
            let lo = (y as f32).max(min_y);
            let hi = ((y + 1) as f32).min(max_y);
            under += (hi - lo).max(0.0);
        }
    }
    (under / (2.0 * HALF.y)).clamp(0.0, 1.0)
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
