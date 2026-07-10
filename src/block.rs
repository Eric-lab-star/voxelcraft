//! Block type definitions and their basic properties.

use bevy::prelude::Color;

/// A single voxel type. `Air` is empty space (not rendered).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Block {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Wood,
    Leaves,
}

impl Block {
    /// Whether this block is solid for meshing/raycasting (culls faces, stops
    /// rays). Water counts as solid here so we don't draw hidden underwater
    /// faces.
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }

    /// Whether this block stops the player. Water is passable so you can wade
    /// and sink into it.
    pub fn blocks_movement(self) -> bool {
        !matches!(self, Block::Air | Block::Water)
    }

    /// Stable numeric id for saving to disk.
    pub fn to_id(self) -> u8 {
        match self {
            Block::Air => 0,
            Block::Grass => 1,
            Block::Dirt => 2,
            Block::Stone => 3,
            Block::Sand => 4,
            Block::Water => 5,
            Block::Wood => 6,
            Block::Leaves => 7,
        }
    }

    /// Inverse of [`Block::to_id`]; unknown ids read as `Air`.
    pub fn from_id(id: u8) -> Block {
        match id {
            1 => Block::Grass,
            2 => Block::Dirt,
            3 => Block::Stone,
            4 => Block::Sand,
            5 => Block::Water,
            6 => Block::Wood,
            7 => Block::Leaves,
            _ => Block::Air,
        }
    }

    /// A representative solid colour, used for break particles.
    pub fn particle_color(self) -> Color {
        match self {
            Block::Air => Color::NONE,
            Block::Grass => Color::srgb(0.35, 0.55, 0.25),
            Block::Dirt => Color::srgb(0.45, 0.32, 0.20),
            Block::Stone => Color::srgb(0.50, 0.50, 0.53),
            Block::Sand => Color::srgb(0.82, 0.76, 0.53),
            Block::Water => Color::srgb(0.20, 0.42, 0.75),
            Block::Wood => Color::srgb(0.40, 0.28, 0.15),
            Block::Leaves => Color::srgb(0.24, 0.48, 0.20),
        }
    }
}
