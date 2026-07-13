//! Break blocks (left click) and place blocks (right click) using a voxel
//! raycast from the camera along its view direction.

use crate::block::Block;
use crate::chunk::{chunk_of, DirtyChunks};
use crate::hotbar::Hotbar;
use crate::particles::{spawn_break_particles, ParticleAssets};
use crate::player::Player;
use crate::water::{disturb, WaterSim};
use crate::world::World;
use bevy::prelude::*;

const REACH: f32 = 8.0;

/// Result of a voxel raycast.
struct RayHit {
    /// The solid block that was hit.
    block: IVec3,
    /// The empty cell just before it (where a new block would be placed).
    prev: IVec3,
}

/// Amanatides & Woo fast voxel traversal. Walks the grid from `origin` along
/// `dir` until it hits a solid block or exceeds `REACH`.
fn raycast(world: &World, origin: Vec3, dir: Vec3) -> Option<RayHit> {
    let dir = dir.normalize_or_zero();
    if dir == Vec3::ZERO {
        return None;
    }

    let mut voxel = origin.floor().as_ivec3();
    let step = IVec3::new(
        dir.x.signum() as i32,
        dir.y.signum() as i32,
        dir.z.signum() as i32,
    );

    // Distance (in t) to cross one voxel along each axis.
    let delta = Vec3::new(
        if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::INFINITY },
        if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::INFINITY },
        if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::INFINITY },
    );

    // Distance (in t) to the first voxel boundary on each axis.
    let mut t_max = Vec3::new(
        boundary_dist(origin.x, dir.x),
        boundary_dist(origin.y, dir.y),
        boundary_dist(origin.z, dir.z),
    );

    let mut prev = voxel;
    let mut t = 0.0;
    while t <= REACH {
        // Pass through air *and* water, stopping only at a real solid block.
        // This lets you aim through water and build on the lakebed underwater.
        let block = world.get(voxel.x, voxel.y, voxel.z);
        if block != Block::Air && block != Block::Water {
            return Some(RayHit { block: voxel, prev });
        }
        prev = voxel;
        // Advance along whichever axis has the nearest boundary.
        if t_max.x < t_max.y && t_max.x < t_max.z {
            voxel.x += step.x;
            t = t_max.x;
            t_max.x += delta.x;
        } else if t_max.y < t_max.z {
            voxel.y += step.y;
            t = t_max.y;
            t_max.y += delta.y;
        } else {
            voxel.z += step.z;
            t = t_max.z;
            t_max.z += delta.z;
        }
    }
    None
}

/// t at which the ray first crosses a voxel boundary along one axis.
fn boundary_dist(origin: f32, dir: f32) -> f32 {
    if dir == 0.0 {
        return f32::INFINITY;
    }
    let frac = origin - origin.floor();
    if dir > 0.0 {
        (1.0 - frac) / dir
    } else {
        (frac / -dir).max(0.0)
    }
}

pub fn edit_blocks(
    buttons: Res<ButtonInput<MouseButton>>,
    player_q: Query<&Player>,
    hotbar: Res<Hotbar>,
    particles: Res<ParticleAssets>,
    mut rng: Local<u32>,
    mut commands: Commands,
    mut world: ResMut<World>,
    mut dirty: ResMut<DirtyChunks>,
    mut water: ResMut<WaterSim>,
) {
    let break_block = buttons.just_pressed(MouseButton::Left);
    let place_block = buttons.just_pressed(MouseButton::Right);
    if !break_block && !place_block {
        return;
    }

    let Ok(player) = player_q.single() else {
        return;
    };

    let Some(hit) = raycast(&world, player.eye(), player.forward()) else {
        return;
    };

    // Break the solid block we hit; place into the cell just before it. That
    // cell may be water — placing there displaces the water, so you can build
    // on the lakebed and stack blocks up through the water.
    let target = if break_block { hit.block } else { hit.prev };

    // Don't let the player wall themselves into a block they're standing in.
    if place_block && player.intersects_block(target) {
        return;
    }

    // Remember what we're breaking so we can spray matching particles.
    let broken = world.get(target.x, target.y, target.z);
    let new_block = if break_block { Block::Air } else { hotbar.block() };
    if world.set(target.x, target.y, target.z, new_block) {
        mark_dirty(&mut dirty, target.x, target.z);
        disturb(&mut water, target); // wake nearby water so it can flow
        if break_block && broken.is_solid() {
            let center = target.as_vec3() + Vec3::splat(0.5);
            spawn_break_particles(&mut commands, &particles, broken, center, &mut rng);
        }
    }
}

/// Draw a wireframe box around the block the player is aiming at.
pub fn highlight_target(
    world: Res<World>,
    player_q: Query<&Player>,
    mut gizmos: Gizmos,
) {
    let Ok(player) = player_q.single() else {
        return;
    };
    if let Some(hit) = raycast(&world, player.eye(), player.forward()) {
        let center = hit.block.as_vec3() + Vec3::splat(0.5);
        gizmos.cube(
            Transform::from_translation(center).with_scale(Vec3::splat(1.003)),
            Color::srgb(0.05, 0.05, 0.05),
        );
    }
}

/// Mark the edited chunk and its four neighbours dirty (edits on a chunk
/// border change the neighbouring chunk's visible faces too).
fn mark_dirty(dirty: &mut DirtyChunks, x: i32, z: i32) {
    for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
        dirty.0.insert(chunk_of(x + dx, z + dz));
    }
}
