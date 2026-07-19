//! Flowing water: a small cellular automaton that makes water fall and cascade
//! toward drops (waterfalls, filling pits, draining when you break a lakebed).
//! Plus the underwater view: a blue screen tint and rising bubbles.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::block::Block;
use crate::chunk::{chunk_of, DirtyChunks};
use crate::player::Player;
use crate::world::{World, WATER_SOURCE};

/// How often the water simulation steps (seconds). Higher = slower flow.
const STEP: f32 = 0.28;

/// How fast the surface ripples march (radians/sec). The crossing waves in
/// `water_field` run at different multiples of this, so it reads as choppy water
/// rather than one texture scrolling past.
const WAVE_SPEED: f32 = 2.2;

const DOWN: IVec3 = IVec3::new(0, -1, 0);
const UP: IVec3 = IVec3::new(0, 1, 0);
const H4: [IVec3; 4] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];
const N6: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
];

// --- Flow simulation -------------------------------------------------------

/// Cells that might still move. Kept small: only edited/disturbed water is here.
#[derive(Resource, Default)]
pub struct WaterSim {
    active: HashSet<IVec3>,
    accum: f32,
    tick: u32,
}

/// Wake up water at and around `p` (called whenever a block is edited).
pub fn disturb(sim: &mut WaterSim, p: IVec3) {
    sim.active.insert(p);
    for d in H4 {
        sim.active.insert(p + d);
    }
    sim.active.insert(p + UP);
    sim.active.insert(p + DOWN);
}

fn get(world: &World, p: IVec3) -> Block {
    world.get(p.x, p.y, p.z)
}

pub fn simulate_water(
    time: Res<Time>,
    mut sim: ResMut<WaterSim>,
    mut world: ResMut<World>,
    mut dirty: ResMut<DirtyChunks>,
) {
    sim.accum += time.delta_secs();
    if sim.accum < STEP {
        return;
    }
    sim.accum = 0.0;
    if sim.active.is_empty() {
        return;
    }
    sim.tick = sim.tick.wrapping_add(1);

    let cells: Vec<IVec3> = sim.active.drain().collect();

    // Double-buffered update (read old, write new) so water spreads one ring per
    // tick — a Minecraft-style flow field where a cell's level is one less than
    // its strongest water neighbour, or 7 when fed from directly above.
    let mut updates: Vec<(IVec3, u8)> = Vec::new();
    for c in &cells {
        let here = get(&world, *c);
        if here != Block::Air && here != Block::Water {
            continue; // solid cell
        }
        let cur = level(&world, *c);
        if cur == WATER_SOURCE {
            continue; // sources are fixed
        }

        let mut supply = 0u8;
        if level(&world, *c + UP) > 0 {
            supply = WATER_SOURCE; // fed from above → falls at full strength
        }
        for d in H4 {
            supply = supply.max(level(&world, *c + d));
        }
        let new = if supply >= 2 { supply - 1 } else { 0 };
        if new != cur {
            updates.push((*c, new));
        }
    }

    let mut next: HashSet<IVec3> = HashSet::new();
    let mut changed: Vec<IVec3> = Vec::new();
    for (c, new) in updates {
        if world.set_water_level(c.x, c.y, c.z, new) {
            changed.push(c);
            for d in N6 {
                next.insert(c + d);
            }
        }
    }
    sim.active = next;

    // Rebuild every chunk column touched (and its neighbours, for border faces).
    let mut dirty_cols: HashSet<(i32, i32)> = HashSet::new();
    for c in changed {
        for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            dirty_cols.insert(chunk_of(c.x + dx, c.z + dz));
        }
    }
    dirty.0.extend(dirty_cols);
}

fn level(world: &World, p: IVec3) -> u8 {
    world.water_level(p.x, p.y, p.z)
}

// --- Surface animation ------------------------------------------------------

/// Roll the water surface by repainting the atlas's water tile each frame with
/// an advancing wave phase.
///
/// The tile can't be scrolled with `uv_transform` the way the clouds are: water
/// samples one cell of the shared block atlas, so sliding its UVs would drag in
/// the neighbouring tiles. Repainting 16x16 pixels is cheap, and it animates the
/// hotbar icon along with the surface for free.
pub fn animate_water(
    time: Res<Time>,
    atlas: Res<crate::texture::BlockAtlas>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Some(mut image) = images.get_mut(&atlas.image) {
        crate::texture::write_water_tile(&mut image, time.elapsed_secs() * WAVE_SPEED);
    }
}

// --- Underwater view (tint + bubbles) --------------------------------------

const BUBBLE_INTERVAL: f32 = 0.06;

#[derive(Resource)]
pub struct WaterFx {
    bubble_mesh: Handle<Mesh>,
    bubble_mat: Handle<StandardMaterial>,
    bubble_accum: f32,
    rng: u32,
}

#[derive(Component)]
pub struct Bubble {
    velocity: Vec3,
    life: f32,
}

/// Full-screen blue tint, shown only while the camera is underwater.
#[derive(Component)]
pub struct UnderwaterOverlay;

pub fn setup_water_fx(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bubble_mesh = meshes.add(Sphere::new(0.5));
    let bubble_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.8, 0.9, 1.0, 0.6),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(WaterFx {
        bubble_mesh,
        bubble_mat,
        bubble_accum: 0.0,
        rng: 0x1234_5678,
    });

    // The tint overlay (hidden until submerged).
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.35, 0.62, 0.42)),
        GlobalZIndex(-1),
        Visibility::Hidden,
        UnderwaterOverlay,
    ));
}

pub fn underwater_effect(
    time: Res<Time>,
    world: Res<World>,
    mut fx: ResMut<WaterFx>,
    player: Query<&Transform, With<Player>>,
    mut overlay: Query<&mut Visibility, With<UnderwaterOverlay>>,
    mut commands: Commands,
) {
    let Ok(eye) = player.single().map(|t| t.translation) else {
        return;
    };
    let submerged = world.get(
        eye.x.floor() as i32,
        eye.y.floor() as i32,
        eye.z.floor() as i32,
    ) == Block::Water;

    for mut vis in &mut overlay {
        *vis = if submerged {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !submerged {
        return;
    }

    // Emit bubbles that rise in front of the player.
    fx.bubble_accum += time.delta_secs();
    while fx.bubble_accum >= BUBBLE_INTERVAL {
        fx.bubble_accum -= BUBBLE_INTERVAL;
        let offset = Vec3::new(
            rand(&mut fx.rng) - 0.5,
            rand(&mut fx.rng) * 0.5,
            rand(&mut fx.rng) - 0.5,
        ) * 1.4;
        let size = 0.03 + rand(&mut fx.rng) * 0.05;
        let velocity = Vec3::new(
            (rand(&mut fx.rng) - 0.5) * 0.4,
            1.0 + rand(&mut fx.rng) * 0.8,
            (rand(&mut fx.rng) - 0.5) * 0.4,
        );
        commands.spawn((
            Mesh3d(fx.bubble_mesh.clone()),
            MeshMaterial3d(fx.bubble_mat.clone()),
            Transform::from_translation(eye + offset).with_scale(Vec3::splat(size)),
            Bubble {
                velocity,
                life: 0.9,
            },
        ));
    }
}

pub fn update_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut Transform, &mut Bubble)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut bubble) in &mut bubbles {
        bubble.life -= dt;
        if bubble.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += bubble.velocity * dt;
    }
}

/// Cheap xorshift RNG -> [0,1).
fn rand(state: &mut u32) -> f32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    (x as f32) / (u32::MAX as f32)
}
