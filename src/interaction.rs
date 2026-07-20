//! Break blocks (left click) and place blocks (right click) using a voxel
//! raycast from the camera along its view direction.

use crate::block::Block;
use crate::chunk::{chunk_of, DirtyChunks};
use crate::hotbar::Hotbar;
use crate::mesh::plant_bounds;
use crate::particles::{spawn_break_particles, ParticleAssets};
use crate::player::Player;
use crate::water::{disturb, WaterSim};
use crate::world::World;
use bevy::prelude::*;

const REACH: f32 = 8.0;

/// Result of a voxel raycast.
pub(crate) struct RayHit {
    /// The solid block that was hit.
    pub(crate) block: IVec3,
    /// The empty cell just before it (where a new block would be placed).
    prev: IVec3,
}

/// Amanatides & Woo fast voxel traversal. Walks the grid from `origin` along
/// `dir` until it hits a solid block or exceeds `REACH`.
pub(crate) fn raycast(world: &World, origin: Vec3, dir: Vec3) -> Option<RayHit> {
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
    // Right-click only places when we're actually holding a block; an empty hand
    // places nothing.
    let place_block = buttons.just_pressed(MouseButton::Right) && hotbar.block().is_some();
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
    //
    // Plants are replaceable: aiming at a flower and placing puts the new block
    // *in* the flower's cell rather than in front of it, as in Minecraft.
    let aimed_at_plant = world.get(hit.block.x, hit.block.y, hit.block.z).is_plant();
    let target = if break_block || (place_block && aimed_at_plant) {
        hit.block
    } else {
        hit.prev
    };

    let held = hotbar.block().unwrap_or(Block::Air);
    if place_block {
        // Don't let the player wall themselves into a block they're standing in.
        // Plants don't collide, so they're exempt — you can stand in a flower.
        if !held.is_plant() && player.intersects_block(target) {
            return;
        }
        // A plant needs earth under it, and a cell that's free to take it.
        let below = world.get(target.x, target.y - 1, target.z);
        let cell = world.get(target.x, target.y, target.z);
        let free = cell == Block::Air || cell.is_plant();
        if held.is_plant() && (!below.supports_plants() || !free) {
            return;
        }
    }

    // Remember what we're breaking so we can spray matching particles.
    let broken = world.get(target.x, target.y, target.z);
    let new_block = if break_block { Block::Air } else { held };
    if world.set(target.x, target.y, target.z, new_block) {
        mark_dirty(&mut dirty, target.x, target.z);
        disturb(&mut water, target); // wake nearby water so it can flow
        // Whatever we just did to this cell may have pulled the ground out from
        // under a plant sitting on it.
        prune_plant_above(&mut world, &mut dirty, target);
        if break_block && broken.is_solid() {
            let center = target.as_vec3() + Vec3::splat(0.5);
            spawn_break_particles(&mut commands, &particles, broken, center, &mut rng);
        }
    }
}

/// Break the plant directly above `p` if `p` can no longer support it. Plants
/// are only ever one block tall, so a single check is enough — there is no chain
/// of dependents to walk.
fn prune_plant_above(world: &mut World, dirty: &mut DirtyChunks, p: IVec3) {
    let above = p + IVec3::Y;
    if !world.get(above.x, above.y, above.z).is_plant() {
        return;
    }
    if world.get(p.x, p.y, p.z).supports_plants() {
        return;
    }
    if world.set(above.x, above.y, above.z, Block::Air) {
        mark_dirty(dirty, above.x, above.z);
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
        // Plants don't fill their cell, so a cell-sized outline would hang in the
        // air around them as a black box. Hug the sprite instead.
        let block = world.get(hit.block.x, hit.block.y, hit.block.z);
        let (center, size) = if block.is_plant() {
            plant_bounds(hit.block, block)
        } else {
            (hit.block.as_vec3() + Vec3::splat(0.5), Vec3::splat(1.003))
        };
        gizmos.cube(
            Transform::from_translation(center).with_scale(size),
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

/// The 푯말 names, looked up by the board's own position.
///
/// Built once at startup from the palace layout. It is safe to hold for every
/// map because a lookup only happens when the ray has already landed on a
/// `Signpost` block, and no other map has any.
#[derive(Resource)]
pub struct SignNames(std::collections::HashMap<IVec3, &'static str>);

pub fn setup_sign_names(mut commands: Commands) {
    commands.insert_resource(SignNames(
        crate::joseon::signposts().into_iter().collect(),
    ));
}

/// Marks the label that names whatever 푯말 you are looking at.
#[derive(Component)]
pub struct SignLabel;

/// The name appears just under the crosshair, where the eye already is when
/// you are aiming at something, rather than at the top of the screen with the
/// toasts — those announce that something happened, this describes what you are
/// looking at now.
pub fn setup_sign_label(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Percent(56.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(crate::font::PIXEL_GRID * 2.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.96, 0.86)),
                SignLabel,
            ));
        });
}

/// Show the name of the 푯말 under the crosshair, and nothing when there isn't
/// one. Written straight from the aim each frame rather than latched on and off,
/// so it cannot get stuck showing a board you have walked away from.
pub fn show_sign_name(
    world: Res<World>,
    names: Res<SignNames>,
    menu: Res<crate::menu::MenuState>,
    player_q: Query<&Player>,
    mut label_q: Query<&mut Text, With<SignLabel>>,
) {
    let Ok(mut text) = label_q.single_mut() else {
        return;
    };
    // Runs even while a menu is up, unlike the rest of the aiming systems,
    // precisely so it can clear itself. Left in the paused group it simply
    // stopped updating, and the last name you happened to be looking at stayed
    // on screen behind the pause overlay.
    let name = (!menu.ui_focused())
        .then(|| player_q
            .single()
            .ok()
            .and_then(|player| raycast(&world, player.eye(), player.forward()))
            .filter(|hit| world.get(hit.block.x, hit.block.y, hit.block.z) == Block::Signpost)
            .and_then(|hit| names.0.get(&hit.block).copied()))
        .flatten()
        .unwrap_or("");
    if text.0 != name {
        text.0 = name.to_string();
    }
}
