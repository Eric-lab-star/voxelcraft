//! A procedurally-generated texture atlas.
//!
//! We paint every block tile in code (16×16 px each) instead of shipping image
//! files — no assets to load, no copyright concerns, and it gives a pleasant
//! hand-made pixel look. Tiles are packed into one atlas image; `mesh.rs` maps
//! each face's UVs into the correct tile.

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const TILE: usize = 16;
pub const COLS: usize = 4;
pub const ROWS: usize = 3;
pub const NUM_TILES: u32 = 9;

// Tile indices within the atlas.
pub const T_GRASS_TOP: u32 = 0;
pub const T_GRASS_SIDE: u32 = 1;
pub const T_DIRT: u32 = 2;
pub const T_STONE: u32 = 3;
pub const T_SAND: u32 = 4;
pub const T_WATER: u32 = 5;
pub const T_WOOD_TOP: u32 = 6;
pub const T_WOOD_SIDE: u32 = 7;
pub const T_LEAVES: u32 = 8;

/// Shared handles for the block atlas: the image plus a grid layout so UI
/// (the hotbar) can address individual tiles by index.
#[derive(Resource)]
pub struct BlockAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Build the atlas image + layout once, before the world/hotbar are set up.
pub fn setup_atlas(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = images.add(build_atlas());
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE as u32),
        COLS as u32,
        ROWS as u32,
        None,
        None,
    ));
    commands.insert_resource(BlockAtlas { image, layout });
}

/// Build the atlas image (COLS×ROWS tiles of TILE×TILE pixels).
pub fn build_atlas() -> Image {
    let w = COLS * TILE;
    let h = ROWS * TILE;
    let mut data = vec![0u8; w * h * 4];

    for tile in 0..NUM_TILES {
        let col = (tile as usize) % COLS;
        let row = (tile as usize) / COLS;
        for y in 0..TILE {
            for x in 0..TILE {
                let px = col * TILE + x;
                let py = row * TILE + y;
                let color = tile_pixel(tile, x, y);
                let i = (py * w + px) * 4;
                data[i..i + 4].copy_from_slice(&color);
            }
        }
    }

    Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

/// Build a seamless cloud texture: white puffs with soft alpha over transparent
/// sky. Tiles across the cloud plane and scrolls to drift.
pub fn build_clouds() -> Image {
    const N: usize = 128;
    let mut data = vec![0u8; N * N * 4];
    for y in 0..N {
        for x in 0..N {
            // Fractal (multi-octave) noise breaks up the regular grid look and
            // gives organic, irregular puffs.
            let fx = x as f32 / N as f32;
            let fy = y as f32 / N as f32;
            let v = 0.6 * cloud_noise(fx, fy, 3)
                + 0.3 * cloud_noise(fx, fy, 6)
                + 0.1 * cloud_noise(fx, fy, 12);
            // Higher threshold => sparser, scattered clouds with open blue sky.
            let strength = smoothstep(0.56, 0.70, v);
            let i = (y * N + x) * 4;
            data[i] = 248;
            data[i + 1] = 250;
            data[i + 2] = 255;
            data[i + 3] = (strength * 210.0) as u8;
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: N as u32,
            height: N as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    // Repeat sampling so the UVs (0..TILES) actually tile the clouds across the
    // sky instead of clamping to a transparent edge.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

/// Wrapping value noise in [0,1) for seamless, tileable clouds.
fn cloud_noise(fx: f32, fy: f32, grid: i32) -> f32 {
    let gx = fx * grid as f32;
    let gy = fy * grid as f32;
    let x0 = gx.floor() as i32;
    let y0 = gy.floor() as i32;
    let sx = smoothstep(0.0, 1.0, gx - x0 as f32);
    let sy = smoothstep(0.0, 1.0, gy - y0 as f32);
    let h = |ix: i32, iy: i32| hash(ix.rem_euclid(grid), iy.rem_euclid(grid));
    let top = lerp(h(x0, y0), h(x0 + 1, y0), sx);
    let bot = lerp(h(x0, y0 + 1), h(x0 + 1, y0 + 1), sx);
    lerp(top, bot, sy)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Build a 32×32 RGBA window/taskbar icon: a grassy block face.
pub fn build_icon() -> (Vec<u8>, u32, u32) {
    const S: usize = 32;
    let mut data = vec![0u8; S * S * 4];
    for y in 0..S {
        for x in 0..S {
            // Upscale the 16px grass-side tile to 32px.
            let c = tile_pixel(T_GRASS_SIDE, x * TILE / S, y * TILE / S);
            let i = (y * S + x) * 4;
            data[i..i + 4].copy_from_slice(&c);
        }
    }
    (data, S as u32, S as u32)
}

/// Paint a single pixel of a tile.
fn tile_pixel(tile: u32, x: usize, y: usize) -> [u8; 4] {
    let n = hash(tile as i32 * 131 + x as i32, y as i32);
    match tile {
        T_GRASS_TOP => shade([88, 148, 58], n, 26.0),
        T_DIRT => dirt(x, y, n),
        T_GRASS_SIDE => {
            // Green turf on the top few rows, dirt below with a ragged edge.
            let edge = 3 + (hash(x as i32, 77) * 2.0) as usize;
            if y < edge {
                shade([88, 148, 58], n, 26.0)
            } else {
                dirt(x, y, n)
            }
        }
        T_STONE => {
            let mut c = shade([124, 124, 130], n, 22.0);
            if hash(x as i32 * 7, y as i32 * 3) > 0.93 {
                c = shade([96, 96, 102], n, 10.0); // dark fleck
            }
            c
        }
        T_SAND => shade([214, 200, 150], n, 20.0),
        T_WATER => {
            // Subtle horizontal ripple.
            let ripple = ((y as f32 * 1.3).sin() * 10.0) as i32;
            shade_offset([48, 108, 190], n, 14.0, ripple)
        }
        T_WOOD_TOP => {
            // Concentric growth rings.
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            let dist = (dx * dx + dy * dy).sqrt();
            if (dist * 1.4) as i32 % 2 == 0 {
                shade([112, 82, 48], n, 12.0)
            } else {
                shade([92, 66, 38], n, 12.0)
            }
        }
        T_WOOD_SIDE => {
            // Vertical bark streaks.
            let streak = hash(x as i32 * 13, 0);
            if streak > 0.6 {
                shade([104, 74, 42], n, 10.0)
            } else {
                shade([84, 58, 32], n, 10.0)
            }
        }
        T_LEAVES => {
            let mut c = shade([54, 108, 44], n, 30.0);
            if hash(x as i32 * 5, y as i32 * 9) > 0.82 {
                c = shade([38, 82, 32], n, 12.0); // darker clumps
            }
            c
        }
        _ => [255, 0, 255, 255], // magenta = missing
    }
}

fn dirt(x: usize, y: usize, n: f32) -> [u8; 4] {
    let mut c = shade([124, 88, 56], n, 18.0);
    if hash(x as i32 * 3, y as i32 * 11) > 0.9 {
        c = shade([98, 68, 42], n, 8.0); // pebble
    }
    c
}

fn shade(base: [u8; 3], n: f32, amp: f32) -> [u8; 4] {
    shade_offset(base, n, amp, 0)
}

fn shade_offset(base: [u8; 3], n: f32, amp: f32, offset: i32) -> [u8; 4] {
    let d = ((n - 0.5) * amp) as i32 + offset;
    [
        (base[0] as i32 + d).clamp(0, 255) as u8,
        (base[1] as i32 + d).clamp(0, 255) as u8,
        (base[2] as i32 + d).clamp(0, 255) as u8,
        255,
    ]
}

/// Deterministic value hash -> [0,1).
fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263))) as u32;
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h % 1000) as f32 / 1000.0
}

/// Which atlas tile a given block shows on a given face.
/// `face` follows the order in `mesh.rs`: 0=-X 1=+X 2=-Y 3=+Y 4=-Z 5=+Z.
pub fn block_tile(block: crate::block::Block, face: usize) -> u32 {
    use crate::block::Block;
    let top = face == 3;
    let bottom = face == 2;
    match block {
        Block::Air => 0,
        Block::Grass => {
            if top {
                T_GRASS_TOP
            } else if bottom {
                T_DIRT
            } else {
                T_GRASS_SIDE
            }
        }
        Block::Dirt => T_DIRT,
        Block::Stone => T_STONE,
        Block::Sand => T_SAND,
        Block::Water => T_WATER,
        Block::Wood => {
            if top || bottom {
                T_WOOD_TOP
            } else {
                T_WOOD_SIDE
            }
        }
        Block::Leaves => T_LEAVES,
    }
}

/// Map a tile-local UV in [0,1] to atlas UV, with a half-texel inset so
/// neighbouring tiles never bleed in under nearest-neighbour sampling.
pub fn atlas_uv(tile: u32, u: f32, v: f32) -> [f32; 2] {
    let col = (tile as usize % COLS) as f32;
    let row = (tile as usize / COLS) as f32;
    let inset = 0.5 / TILE as f32;
    let u = inset + u * (1.0 - 2.0 * inset);
    let v = inset + v * (1.0 - 2.0 * inset);
    [(col + u) / COLS as f32, (row + v) / ROWS as f32]
}
