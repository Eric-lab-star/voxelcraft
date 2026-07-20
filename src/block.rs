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
    TallGrass,
    RedFlower,
    YellowFlower,
    /// 기와 — the dark clay roof tile of a hanok.
    RoofTile,
    /// 회벽 — the pale lime-plaster infill between a hanok's timber posts.
    Plaster,
    /// 한지 창호 — a paper-and-lattice door panel.
    Paper,
    /// 단청 — the polychrome painted band on palace beams. More than any other
    /// single thing, this is what makes a building read as Korean rather than
    /// generically East Asian.
    Dancheong,
    /// 붉은 기둥 — the vermilion-lacquered column of a palace hall.
    RedPillar,
    /// 용마루 — the white lime-plastered ridge line (양성바름) that caps a palace
    /// roof, and the pale tile of its swept eaves.
    RoofRidge,
    /// 화강암 — dressed granite ashlar, for the 월대 terraces and palace walls.
    Granite,
    /// 초가지붕 — the straw thatch of a commoner's house. Setting these against
    /// the tiled roofs of the well-off is what gives a Joseon village its mix.
    Thatch,
    /// 흙담 — a mud-and-straw wall, for commoners' houses and courtyard walls.
    ClayWall,
    /// 흙길 — the beaten-earth surface of a street.
    Road,
    /// 푯말 — a board on a post, standing in front of a building. Looking at
    /// one names the building; see `joseon::signpost_name`.
    Signpost,
}

impl Block {
    /// Whether this block is solid for meshing/raycasting (culls faces, stops
    /// rays). Water counts as solid here so we don't draw hidden underwater
    /// faces.
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }

    /// Whether this block stops the player. Water is passable so you can wade
    /// and sink into it; plants are decoration you walk straight through.
    pub fn blocks_movement(self) -> bool {
        !matches!(self, Block::Air | Block::Water) && !self.is_plant()
    }

    /// Whether this is a flat cross-shaped decoration (grass tuft, flower)
    /// rather than a cube. Plants don't fill their cell: they cull nothing, stop
    /// nothing, need a block underneath to stand on, and anything placed into
    /// their cell — a block, or spreading water — simply replaces them.
    pub fn is_plant(self) -> bool {
        matches!(
            self,
            Block::TallGrass | Block::RedFlower | Block::YellowFlower
        )
    }

    /// Whether a plant can root on top of this block.
    pub fn supports_plants(self) -> bool {
        matches!(self, Block::Grass | Block::Dirt | Block::Sand)
    }

    /// How tall this plant stands, in blocks. Decoration reads better well under
    /// a full block: at 1.0 a tuft is as tall as the terrain step beside it and
    /// stops looking like ground cover.
    pub fn plant_height(self) -> f32 {
        match self {
            Block::TallGrass => 0.7,
            Block::RedFlower | Block::YellowFlower => 0.6,
            _ => 1.0,
        }
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
            Block::TallGrass => 8,
            Block::RedFlower => 9,
            Block::YellowFlower => 10,
            Block::RoofTile => 11,
            Block::Plaster => 12,
            Block::Paper => 13,
            Block::Dancheong => 14,
            Block::RedPillar => 15,
            Block::RoofRidge => 16,
            Block::Granite => 17,
            Block::Thatch => 18,
            Block::ClayWall => 19,
            Block::Road => 20,
            Block::Signpost => 21,
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
            8 => Block::TallGrass,
            9 => Block::RedFlower,
            10 => Block::YellowFlower,
            11 => Block::RoofTile,
            12 => Block::Plaster,
            13 => Block::Paper,
            14 => Block::Dancheong,
            15 => Block::RedPillar,
            16 => Block::RoofRidge,
            17 => Block::Granite,
            18 => Block::Thatch,
            19 => Block::ClayWall,
            20 => Block::Road,
            21 => Block::Signpost,
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
            Block::TallGrass => Color::srgb(0.34, 0.58, 0.22),
            Block::RedFlower => Color::srgb(0.72, 0.20, 0.20),
            Block::YellowFlower => Color::srgb(0.88, 0.75, 0.22),
            Block::RoofTile => Color::srgb(0.28, 0.30, 0.35),
            Block::Plaster => Color::srgb(0.85, 0.82, 0.74),
            Block::Paper => Color::srgb(0.90, 0.84, 0.68),
            Block::Dancheong => Color::srgb(0.22, 0.45, 0.32),
            Block::RedPillar => Color::srgb(0.62, 0.20, 0.16),
            Block::RoofRidge => Color::srgb(0.86, 0.85, 0.82),
            Block::Granite => Color::srgb(0.68, 0.66, 0.62),
            Block::Thatch => Color::srgb(0.78, 0.64, 0.32),
            Block::ClayWall => Color::srgb(0.66, 0.55, 0.40),
            Block::Road => Color::srgb(0.54, 0.46, 0.36),
            Block::Signpost => Color::srgb(0.62, 0.48, 0.30),
        }
    }
}
