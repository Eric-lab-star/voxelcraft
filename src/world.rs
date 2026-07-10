//! The voxel world: a flat 3D grid of blocks plus procedural generation.
//!
//! For a starter we keep the whole map in one contiguous array. It is still
//! *meshed* in chunk-sized columns (see `mesh.rs`), so the structure already
//! points toward real, independently-rebuilt chunks.

use crate::block::Block;
use bevy::math::Vec3;
use bevy::prelude::Resource;
use noise::{NoiseFn, Perlin};

/// Width/depth of a mesh chunk in blocks.
pub const CHUNK_SIZE: i32 = 16;

/// World size in blocks.
pub const WORLD_X: i32 = 256;
pub const WORLD_Z: i32 = 256;
pub const WORLD_Y: i32 = 64;

/// Sea level in blocks.
pub const SEA_LEVEL: i32 = 24;

/// How far in from the world edge the player is walled off. The terrain in this
/// margin is still generated and drawn, so the world edge reads as distant land
/// you can see but can't reach — no abrupt cliff at your feet.
pub const PLAY_MARGIN: i32 = 32;

/// Water level of a full "source" block. Flowing water decreases one step per
/// block away from a source (so it spreads `WATER_SOURCE - 1` blocks); 0 means
/// no water. Lower value = smaller puddles.
pub const WATER_SOURCE: u8 = 5;

#[derive(Resource)]
pub struct World {
    blocks: Vec<Block>,
    /// Per-cell water level, parallel to `blocks`. 0 for non-water.
    levels: Vec<u8>,
}

impl World {
    /// Generate a fresh world with rolling hills, water, beaches, and a few trees.
    pub fn generate(seed: u32) -> Self {
        let n = (WORLD_X * WORLD_Y * WORLD_Z) as usize;
        let mut world = World {
            blocks: vec![Block::Air; n],
            levels: vec![0; n],
        };

        let terrain = Perlin::new(seed);
        let detail = Perlin::new(seed.wrapping_add(1));

        for z in 0..WORLD_Z {
            for x in 0..WORLD_X {
                // Layered noise -> a height value around SEA_LEVEL.
                let nx = x as f64 / 48.0;
                let nz = z as f64 / 48.0;
                let base = terrain.get([nx, nz]) * 10.0;
                let fine = detail.get([nx * 3.0, nz * 3.0]) * 3.0;
                let height = (SEA_LEVEL as f64 + base + fine).round() as i32;
                let height = height.clamp(1, WORLD_Y - 1);

                for y in 0..=height {
                    let block = if y == height {
                        if height <= SEA_LEVEL + 1 {
                            Block::Sand
                        } else {
                            Block::Grass
                        }
                    } else if y >= height - 3 {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    world.set(x, y, z, block);
                }

                // Fill open water up to sea level.
                for y in (height + 1)..=SEA_LEVEL {
                    world.set(x, y, z, Block::Water);
                }

                // Sparse trees on grass above the waterline.
                if height > SEA_LEVEL + 1
                    && world.get(x, height, z) == Block::Grass
                    && pseudo_random(x, z, seed) < 0.02
                {
                    world.place_tree(x, height + 1, z);
                }
            }
        }

        world
    }

    fn place_tree(&mut self, x: i32, base_y: i32, z: i32) {
        let trunk = 4;
        for i in 0..trunk {
            self.set(x, base_y + i, z, Block::Wood);
        }
        let top = base_y + trunk;
        // A small blob of leaves around the top of the trunk.
        for dy in -1..=1 {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if dx * dx + dz * dz + dy * dy <= 5 {
                        let (lx, ly, lz) = (x + dx, top + dy, z + dz);
                        if self.get(lx, ly, lz) == Block::Air {
                            self.set(lx, ly, lz, Block::Leaves);
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn index(x: i32, y: i32, z: i32) -> Option<usize> {
        if x < 0 || y < 0 || z < 0 || x >= WORLD_X || y >= WORLD_Y || z >= WORLD_Z {
            return None;
        }
        Some((x + z * WORLD_X + y * WORLD_X * WORLD_Z) as usize)
    }

    /// Read a block. Anything outside the world reads as `Air`.
    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> Block {
        match Self::index(x, y, z) {
            Some(i) => self.blocks[i],
            None => Block::Air,
        }
    }

    /// Serialise the world to a simple `VOX1` file (magic + dims + one byte
    /// per block).
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(self.blocks.len() + 16);
        buf.extend_from_slice(b"VOX1");
        buf.extend_from_slice(&WORLD_X.to_le_bytes());
        buf.extend_from_slice(&WORLD_Y.to_le_bytes());
        buf.extend_from_slice(&WORLD_Z.to_le_bytes());
        buf.extend(self.blocks.iter().map(|b| b.to_id()));
        std::fs::write(path, buf)
    }

    /// Load a world saved by [`World::save`]. Returns `None` if the file is
    /// missing, corrupt, or was made with different world dimensions.
    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 16 || &data[0..4] != b"VOX1" {
            return None;
        }
        let dims = |o: usize| i32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        if dims(4) != WORLD_X || dims(8) != WORLD_Y || dims(12) != WORLD_Z {
            return None;
        }
        let body = &data[16..];
        if body.len() != (WORLD_X * WORLD_Y * WORLD_Z) as usize {
            return None;
        }
        let blocks: Vec<Block> = body.iter().map(|&b| Block::from_id(b)).collect();
        // Treat all saved water as full source blocks.
        let levels = blocks
            .iter()
            .map(|b| if *b == Block::Water { WATER_SOURCE } else { 0 })
            .collect();
        Some(World { blocks, levels })
    }

    /// Water level at a cell (0 outside the world / for non-water).
    #[inline]
    pub fn water_level(&self, x: i32, y: i32, z: i32) -> u8 {
        match Self::index(x, y, z) {
            Some(i) => self.levels[i],
            None => 0,
        }
    }

    /// Set a cell's water level directly (used by the flow simulation). Only
    /// affects water/air cells; solids are left alone. `level == 0` clears the
    /// water back to air.
    pub fn set_water_level(&mut self, x: i32, y: i32, z: i32, level: u8) -> bool {
        match Self::index(x, y, z) {
            Some(i) => {
                let b = self.blocks[i];
                if b != Block::Water && b != Block::Air {
                    return false;
                }
                let new_block = if level > 0 { Block::Water } else { Block::Air };
                let changed = self.levels[i] != level || self.blocks[i] != new_block;
                self.levels[i] = level;
                self.blocks[i] = new_block;
                changed
            }
            None => false,
        }
    }

    /// Highest movement-blocking block in a column (its Y), or -1 if none.
    pub fn surface_y(&self, x: i32, z: i32) -> i32 {
        for y in (0..WORLD_Y).rev() {
            if self.get(x, y, z).blocks_movement() {
                return y;
            }
        }
        -1
    }

    /// Find a pleasant spawn: the nearest grass column to the map centre whose
    /// surface sits above the waterline. Returns the player box *centre*.
    pub fn find_spawn(&self) -> Vec3 {
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        for r in 0..(WORLD_X / 2) {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dz.abs() != r {
                        continue; // only the outer ring at this radius
                    }
                    let (x, z) = (cx + dx, cz + dz);
                    let sy = self.surface_y(x, z);
                    if sy > SEA_LEVEL && self.get(x, sy, z) == Block::Grass {
                        // Feet on top of the surface block; centre is 0.9 above.
                        return Vec3::new(x as f32 + 0.5, sy as f32 + 1.9, z as f32 + 0.5);
                    }
                }
            }
        }
        Vec3::new(cx as f32 + 0.5, WORLD_Y as f32, cz as f32 + 0.5)
    }

    /// Write a block. Out-of-bounds writes are ignored. Returns whether the
    /// world actually changed. Placing water makes a full source block; any
    /// other block clears the cell's water level.
    pub fn set(&mut self, x: i32, y: i32, z: i32, block: Block) -> bool {
        match Self::index(x, y, z) {
            Some(i) => {
                let new_level = if block == Block::Water { WATER_SOURCE } else { 0 };
                let changed = self.blocks[i] != block || self.levels[i] != new_level;
                self.blocks[i] = block;
                self.levels[i] = new_level;
                changed
            }
            None => false,
        }
    }
}

/// Cheap deterministic hash -> [0,1). Used to scatter trees without an RNG.
fn pseudo_random(x: i32, z: i32, seed: u32) -> f64 {
    let mut h = seed as u64;
    h = h.wrapping_mul(6364136223846793005).wrapping_add(x as u64);
    h = h.wrapping_mul(6364136223846793005).wrapping_add(z as u64);
    h ^= h >> 33;
    (h % 10_000) as f64 / 10_000.0
}
