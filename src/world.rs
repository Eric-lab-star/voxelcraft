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
///
/// Driven by the palace and by the ground around it. 경복궁's precinct is now
/// 336 by 455, and the two directions that matter outside it are north, where
/// 북악산 stands immediately behind the palace, and south, where 육조거리 runs
/// away from 광화문. 768 leaves 98 blocks of walkable ground to the north and
/// 151 to the south.
///
/// Costs, measured on the 조선 map in release: worldgen 351ms and meshing all
/// 2304 columns 1.61s, so about two seconds of startup — behind the title
/// screen, which is where the world gets built anyway. Memory is 151MB
/// resident, two bytes a cell.
///
/// A save slot is 220KB on the 조선 map and 2MB on the meadow, the format being
/// run-length encoded.
pub const WORLD_X: i32 = 768;
pub const WORLD_Z: i32 = 768;
/// Vertical extent. At 64 the palace had 31 blocks of headroom above
/// `FLAT_LEVEL` — 근정전 already used 18 of them, which left no room to build
/// its halls at a size where 공포 brackets and a swept 처마 are more than one
/// block each. 128 buys that room, and the ridge line behind the palace later.
pub const WORLD_Y: i32 = 128;

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

/// Which map to generate. Each is a whole landscape recipe, not a biome inside
/// one world — picking one regenerates everything.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MapKind {
    /// Rolling hills, lakes, oaks and wildflowers — the original world.
    Meadow,
    /// 조선 — ridged Korean mountains, terraced rice paddies, pine woods and a
    /// hanok village.
    Joseon,
    /// 빈 터 — a bare level plain. Nothing is generated on it at all: no trees,
    /// no plants, no water, no buildings. A blank canvas to build on.
    Flat,
}

impl MapKind {
    pub fn label(self) -> &'static str {
        match self {
            MapKind::Meadow => "초원",
            MapKind::Joseon => "조선",
            MapKind::Flat => "빈 터",
        }
    }
}

/// Ground level of the [`MapKind::Flat`] map. Deliberately above `SEA_LEVEL` so
/// the plain is dry land and [`World::find_spawn`] can stand you on it.
pub const FLAT_LEVEL: i32 = SEA_LEVEL + 8;

#[derive(Resource)]
pub struct World {
    blocks: Vec<Block>,
    /// Per-cell water level, parallel to `blocks`. 0 for non-water.
    levels: Vec<u8>,
}

impl World {
    /// An empty world of the standard dimensions, ready to be filled in.
    pub fn empty() -> Self {
        let n = (WORLD_X * WORLD_Y * WORLD_Z) as usize;
        World {
            blocks: vec![Block::Air; n],
            levels: vec![0; n],
        }
    }

    /// Generate a fresh world of the given kind.
    pub fn generate(kind: MapKind, seed: u32) -> Self {
        match kind {
            MapKind::Meadow => Self::generate_meadow(seed),
            MapKind::Joseon => crate::joseon::generate(seed),
            MapKind::Flat => Self::generate_flat(),
        }
    }

    /// A featureless level plain: stone, a little dirt, grass on top. No
    /// `decorate` pass and no terrain noise — the point is that nothing is here
    /// but the ground you build on.
    fn generate_flat() -> Self {
        let mut world = World::empty();
        world.fill_flat(FLAT_LEVEL);
        world
    }

    /// Fill the whole map with a level plain topped out at `level`. Shared by
    /// the blank map and the Joseon map, which both build on flat ground.
    pub fn fill_flat(&mut self, level: i32) {
        for z in 0..WORLD_Z {
            for x in 0..WORLD_X {
                for y in 0..=level {
                    let block = if y == level {
                        Block::Grass
                    } else if y >= level - 3 {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    self.set(x, y, z, block);
                }
            }
        }
    }

    /// Rolling hills, water, beaches, and a few trees.
    fn generate_meadow(seed: u32) -> Self {
        let mut world = World::empty();

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

        world.decorate(seed);
        world
    }

    /// Scatter grass tufts and flowers over any bare grass. Runs after the
    /// terrain and trees, and only ever writes into `Air` directly above a
    /// supporting block, so it can never carve into terrain, a tree, or anything
    /// the player has built — which also makes it safe to run on a world loaded
    /// from an older save that predates plants.
    pub fn decorate(&mut self, seed: u32) {
        for z in 0..WORLD_Z {
            for x in 0..WORLD_X {
                let y = self.surface_y(x, z);
                if y < 0 || y >= WORLD_Y - 1 {
                    continue;
                }
                // Grass only, and only above the waterline: no flowers on the
                // seabed or poking out of a lake.
                if self.get(x, y, z) != Block::Grass || y <= SEA_LEVEL {
                    continue;
                }
                if self.get(x, y + 1, z) != Block::Air {
                    continue;
                }

                // Two independent rolls, so flowers aren't merely rare grass:
                // they thin out in their own patches.
                let r = pseudo_random(x, z, seed.wrapping_add(101));
                let block = if r < 0.03 {
                    if pseudo_random(x, z, seed.wrapping_add(202)) < 0.5 {
                        Block::RedFlower
                    } else {
                        Block::YellowFlower
                    }
                } else if r < 0.28 {
                    Block::TallGrass
                } else {
                    continue;
                };
                self.set(x, y + 1, z, block);
            }
        }
    }

    /// Does the world contain any plants at all? Used once at startup to decide
    /// whether a loaded save predates them and wants decorating.
    pub fn has_plants(&self) -> bool {
        self.blocks.iter().any(|b| b.is_plant())
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

    /// Serialise the world to a `VOX2` file: magic, dimensions, then the blocks
    /// run-length encoded as `(count: u16, id: u8)` pairs.
    ///
    /// Raw, a slot was 72MB — three-quarters of a gigabyte across the save
    /// slots — and these worlds are almost entirely air above a flat plain, so
    /// nearly all of that was the same byte repeated. Runs cap at `u16::MAX` and
    /// simply continue in the next pair, which costs three bytes per 65535 and
    /// saves writing a variable-length integer.
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1 << 16);
        buf.extend_from_slice(b"VOX2");
        buf.extend_from_slice(&WORLD_X.to_le_bytes());
        buf.extend_from_slice(&WORLD_Y.to_le_bytes());
        buf.extend_from_slice(&WORLD_Z.to_le_bytes());

        let mut push = |id: u8, count: u32| {
            let mut left = count;
            while left > 0 {
                let n = left.min(u16::MAX as u32);
                buf.extend_from_slice(&(n as u16).to_le_bytes());
                buf.push(id);
                left -= n;
            }
        };
        let mut run_id = self.blocks[0].to_id();
        let mut run_len: u32 = 0;
        for b in &self.blocks {
            let id = b.to_id();
            if id == run_id {
                run_len += 1;
            } else {
                push(run_id, run_len);
                run_id = id;
                run_len = 1;
            }
        }
        push(run_id, run_len);
        std::fs::write(path, buf)
    }

    /// Load a world saved by [`World::save`]. Returns `None` if the file is
    /// missing, corrupt, or was made with different world dimensions.
    pub fn load(path: &str) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        if data.len() < 16 || &data[0..4] != b"VOX2" {
            return None;
        }
        let dims = |o: usize| i32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        if dims(4) != WORLD_X || dims(8) != WORLD_Y || dims(12) != WORLD_Z {
            return None;
        }
        let cells = (WORLD_X * WORLD_Y * WORLD_Z) as usize;
        let body = &data[16..];
        if body.len() % 3 != 0 {
            return None;
        }
        let mut blocks: Vec<Block> = Vec::with_capacity(cells);
        for pair in body.chunks_exact(3) {
            let n = u16::from_le_bytes([pair[0], pair[1]]) as usize;
            // A corrupt file must not be able to make us allocate the world
            // several times over before we notice.
            if blocks.len() + n > cells {
                return None;
            }
            blocks.extend(std::iter::repeat_n(Block::from_id(pair[2]), n));
        }
        if blocks.len() != cells {
            return None;
        }
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
    /// affects water/air/plant cells; solids are left alone. `level == 0` clears
    /// the water back to air.
    ///
    /// Water washing over a plant destroys it, as in Minecraft. Without this the
    /// flow would treat a flower as a wall and refuse to spread through it.
    pub fn set_water_level(&mut self, x: i32, y: i32, z: i32, level: u8) -> bool {
        match Self::index(x, y, z) {
            Some(i) => {
                let b = self.blocks[i];
                if b != Block::Water && b != Block::Air && !b.is_plant() {
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
        // The palace has a front door, and arriving anywhere else wastes it:
        // the whole map is arranged along one axis that starts at 광화문. This
        // returns `None` on every other map.
        if let Some(gate) = crate::joseon::approach_spawn(self) {
            return gate;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A save must come back exactly as it went in.
    ///
    /// Run-length encoding is the kind of change that can lose a block here and
    /// there and still load something that looks broadly right — an off-by-one
    /// on a run boundary shifts every cell after it, and a palace rebuilt one
    /// block out of place is not obviously wrong until you walk into a wall.
    #[test]
    fn a_save_round_trips_exactly() {
        let before = World::generate(MapKind::Joseon, 7);
        let path = std::env::temp_dir().join("voxelcraft-roundtrip.sav");
        let path = path.to_str().unwrap();
        before.save(path).expect("save failed");
        let after = World::load(path).expect("load failed");
        let _ = std::fs::remove_file(path);

        for z in (0..WORLD_Z).step_by(3) {
            for x in (0..WORLD_X).step_by(3) {
                for y in 0..WORLD_Y {
                    assert_eq!(
                        before.get(x, y, z),
                        after.get(x, y, z),
                        "block changed at ({x},{y},{z})"
                    );
                }
            }
        }
    }

    /// Truncated or scrambled files must be refused, not half-loaded.
    #[test]
    fn a_corrupt_save_is_refused() {
        let w = World::generate(MapKind::Flat, 1);
        let path = std::env::temp_dir().join("voxelcraft-corrupt.sav");
        let path = path.to_str().unwrap();
        w.save(path).unwrap();
        let good = std::fs::read(path).unwrap();

        // Cut a run out of the middle: the block count no longer adds up.
        let mut short = good.clone();
        short.truncate(good.len() - 3);
        std::fs::write(path, &short).unwrap();
        assert!(World::load(path).is_none(), "a short file loaded");

        // Claim runs that overrun the world several times over.
        let mut huge = good[..16].to_vec();
        for _ in 0..64 {
            huge.extend_from_slice(&u16::MAX.to_le_bytes());
            huge.push(0);
        }
        std::fs::write(path, &huge).unwrap();
        assert!(World::load(path).is_none(), "an overlong file loaded");
        let _ = std::fs::remove_file(path);
    }

    /// The flat map is a blank canvas: ground and nothing else. No decorate
    /// pass, no trees, no water — if any of those ever leak into it, the point
    /// of the map is gone.
    #[test]
    fn flat_map_is_empty() {
        let w = World::generate(MapKind::Flat, 1);
        for z in (0..WORLD_Z).step_by(7) {
            for x in (0..WORLD_X).step_by(7) {
                for y in 0..WORLD_Y {
                    let b = w.get(x, y, z);
                    let expected = if y > FLAT_LEVEL {
                        Block::Air
                    } else if y == FLAT_LEVEL {
                        Block::Grass
                    } else if y >= FLAT_LEVEL - 3 {
                        Block::Dirt
                    } else {
                        Block::Stone
                    };
                    assert_eq!(b, expected, "unexpected {b:?} at ({x},{y},{z})");
                }
            }
        }
        // And you have to be able to stand on it.
        let spawn = w.find_spawn();
        assert_eq!(spawn.y, FLAT_LEVEL as f32 + 1.9, "spawn is not on the plain");
    }
}

/// Cheap deterministic hash -> [0,1). Used to scatter trees and plants without
/// an RNG.
///
/// The mix matters more than it looks. An LCG-style `h = h*K + x; h = h*K + z`
/// leaves `z` barely stirred into the low bits, and `% 10_000` reads exactly
/// those — so for a fixed `x` the result marched smoothly with `z` and every
/// scatter came out in vertical stripes. Multiplying each coordinate by its own
/// large odd constant and avalanching afterwards decorrelates the axes.
pub(crate) fn pseudo_random(x: i32, z: i32, seed: u32) -> f64 {
    let mut h = (x as u32)
        .wrapping_mul(374761393)
        ^ (z as u32).wrapping_mul(668265263)
        ^ seed.wrapping_mul(2654435761);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^= h >> 16;
    (h % 10_000) as f64 / 10_000.0
}
