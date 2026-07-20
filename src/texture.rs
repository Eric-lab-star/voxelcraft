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
pub const ROWS: usize = 6;
pub const NUM_TILES: u32 = 22;

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
// Plant tiles. Unlike every tile above, these are mostly *transparent* — they
// are drawn on cross-shaped quads and rely on alpha cutout for their silhouette.
pub const T_TALL_GRASS: u32 = 9;
pub const T_FLOWER_RED: u32 = 10;
pub const T_FLOWER_YELLOW: u32 = 11;
// Hanok building materials.
pub const T_ROOF_TILE: u32 = 12;
pub const T_PLASTER: u32 = 13;
pub const T_PAPER: u32 = 14;
// Palace materials.
pub const T_DANCHEONG: u32 = 15;
pub const T_RED_PILLAR: u32 = 16;
pub const T_ROOF_RIDGE: u32 = 17;
pub const T_GRANITE: u32 = 18;
// Village materials.
pub const T_THATCH: u32 = 19;
pub const T_CLAY_WALL: u32 = 20;
pub const T_ROAD: u32 = 21;

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
        T_WATER => water_pixel(x, y, n, 0.0),
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
        T_TALL_GRASS => tall_grass_pixel(x, y, n),
        // Poppy: dark eye, red petals. Dandelion: pale eye, yellow petals.
        T_FLOWER_RED => flower_pixel(x, y, n, [198, 52, 48], [62, 40, 34]),
        T_FLOWER_YELLOW => flower_pixel(x, y, n, [230, 190, 50], [248, 228, 116]),
        T_ROOF_TILE => roof_tile_pixel(x, y, n),
        T_PLASTER => {
            // Lime plaster: near-white with a faint warm mottle, no pattern —
            // it has to read as flat next to the busy roof and lattice tiles.
            shade([206, 198, 182], n, 14.0)
        }
        T_PAPER => paper_pixel(x, y, n),
        T_DANCHEONG => dancheong_pixel(x, y, n),
        T_RED_PILLAR => red_pillar_pixel(x, y, n),
        T_ROOF_RIDGE => roof_ridge_pixel(x, y, n),
        T_GRANITE => granite_pixel(x, y, n),
        T_THATCH => thatch_pixel(x, y, n),
        T_CLAY_WALL => clay_wall_pixel(x, y, n),
        T_ROAD => road_pixel(x, y, n),
        _ => [255, 0, 255, 255], // magenta = missing
    }
}

/// One pixel of 초가 thatch: bundles of straw laid in overlapping courses. The
/// per-column length jitter is what keeps a roof of these from looking like
/// corduroy — real thatch has a ragged, uneven lower edge to every course.
fn thatch_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    // Courses about 6 px deep, with the boundary wobbling column to column.
    let wobble = (hash(x as i32 * 19, 3) * 2.5) as usize;
    let depth = (y + wobble) % 6;
    // The bottom of each course is in shadow under the one that overlaps it.
    let base = match depth {
        0 => [104, 78, 34],
        1 => [150, 116, 52],
        4 => [128, 98, 44],
        5 => [112, 86, 38],
        _ => [178, 144, 68],
    };
    // A few pale straws catching the light.
    if hash(x as i32 * 7, y as i32 * 5) > 0.9 {
        return shade([206, 176, 100], n, 10.0);
    }
    shade(base, n, 18.0)
}

/// One pixel of 흙담: mud render over a straw binder, with the straw showing
/// through as short pale flecks.
fn clay_wall_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    let mut c = shade([172, 142, 104], n, 20.0);
    // Short horizontal straw flecks, not round speckle — they read as fibres.
    if hash((x / 2) as i32 * 11, y as i32 * 17) > 0.86 {
        c = shade([200, 176, 128], n, 12.0);
    }
    // Occasional darker patch where the render has weathered thin.
    if hash(x as i32 * 5, (y / 3) as i32 * 23) > 0.93 {
        c = shade([142, 116, 84], n, 10.0);
    }
    c
}

/// One pixel of a beaten-earth street: compacted dirt with pebbles pressed into
/// it. Deliberately greyer and flatter than `T_DIRT`, so a road reads as a road
/// where it crosses bare ground.
fn road_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    let mut c = shade([138, 118, 92], n, 16.0);
    if hash(x as i32 * 13, y as i32 * 7) > 0.88 {
        c = shade([164, 156, 146], n, 12.0); // pebble
    } else if hash(x as i32 * 3, y as i32 * 29) > 0.9 {
        c = shade([112, 96, 74], n, 10.0); // rut
    }
    c
}

/// One pixel of 단청: the painted band that runs along a palace beam.
///
/// Real dancheong puts an elaborate 머리초 motif at each end of a beam, which a
/// single repeating 16px tile cannot carry. What survives the reduction — and
/// what the eye actually reads from across a courtyard — is the colour order:
/// a deep green ground with a red-white-blue band running through it.
fn dancheong_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    const GREEN: [u8; 3] = [42, 104, 72];
    const CREAM: [u8; 3] = [230, 224, 204];
    const RED: [u8; 3] = [166, 50, 40];
    const BLUE: [u8; 3] = [48, 84, 138];

    // Dark rails top and bottom, so stacked beams stay visually separated.
    if y == 0 || y == TILE - 1 {
        return shade([38, 40, 44], n, 8.0);
    }
    if (5..=10).contains(&y) {
        // Cream keylines close the band off above and below.
        if y == 5 || y == 10 {
            return shade(CREAM, n, 8.0);
        }
        // The motif repeats twice across the tile.
        return match x % 8 {
            0..=2 => shade(RED, n, 12.0),
            3..=4 => shade(CREAM, n, 10.0),
            _ => shade(BLUE, n, 12.0),
        };
    }
    shade(GREEN, n, 14.0)
}

/// One pixel of a vermilion palace column. The across-the-width shading is what
/// makes a stack of these read as a *round* pillar instead of a flat red post.
/// (`_y` is unused on purpose: a column looks the same all the way up, so the
/// tile only varies across its width.)
fn red_pillar_pixel(x: usize, _y: usize, n: f32) -> [u8; 4] {
    let curve = 1.0 - ((x as f32 - 7.5).abs() / 8.0);
    let base = [
        lerp(112.0, 178.0, curve) as u8,
        lerp(34.0, 62.0, curve) as u8,
        lerp(28.0, 46.0, curve) as u8,
    ];
    // Faint vertical grain in the lacquer.
    let grain = if hash(x as i32 * 23, 5) > 0.7 { -8.0 } else { 0.0 };
    shade(base, n + grain / 255.0, 10.0)
}

/// One pixel of the white-plastered ridge (양성바름). Dark tile courses cap it
/// above and below, so a ridge line reads as a bright band edged in slate — the
/// detail that most distinguishes a palace roof from a commoner's.
fn roof_ridge_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    if y < 2 || y >= TILE - 2 {
        return shade([52, 56, 66], n, 10.0);
    }
    let mut c = shade([224, 220, 210], n, 12.0);
    // A little weathering so a long ridge isn't a flat white stripe.
    if hash(x as i32 * 11, y as i32 * 7) > 0.88 {
        c = shade([198, 194, 184], n, 8.0);
    }
    c
}

/// One pixel of dressed granite ashlar: pale speckled stone cut into courses,
/// with the joints offset row to row the way real coursed masonry is laid.
fn granite_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    let course = y / 8;
    // Offset alternate courses so the vertical joints don't line up into a grid.
    let joint_x = (x + course * 4) % 8 == 0;
    if y % 8 == 0 || joint_x {
        return shade([132, 130, 126], n, 10.0);
    }
    let mut c = shade([176, 174, 168], n, 16.0);
    if hash(x as i32 * 3, y as i32 * 13) > 0.9 {
        c = shade([150, 148, 146], n, 8.0); // mica fleck
    }
    c
}

/// One pixel of a 기와 roof tile: rows of half-round clay tiles running down the
/// slope. The dark seams between them are what make a roof read as tiled rather
/// than as a flat grey slab when you see it from across a valley.
fn roof_tile_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    // Four convex tiles across the width, each 4 px.
    let within = x % 4;
    let base = match within {
        0 => [44, 48, 58],   // shaded valley where two tiles meet
        1 => [92, 98, 112],  // rising face
        2 => [116, 122, 138], // crown, catching the light
        _ => [70, 76, 90],   // falling face
    };
    // Courses overlap every 5 px down the slope; the lip casts a dark line.
    if y % 5 == 0 {
        return shade([34, 38, 46], n, 8.0);
    }
    shade(base, n, 12.0)
}

/// One pixel of a 한지 door: warm paper behind a dark wooden lattice.
fn paper_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    let (fx, fy) = (x as i32, y as i32);
    // Outer frame plus a grid of muntins.
    let frame = fx <= 0 || fy <= 0 || fx >= TILE as i32 - 1 || fy >= TILE as i32 - 1;
    let lattice = fx % 5 == 0 || fy % 5 == 0;
    if frame || lattice {
        return shade([96, 68, 42], n, 10.0);
    }
    // Paper glows warm; the grain is deliberately subtle so the lattice reads.
    shade([226, 208, 166], n, 10.0)
}

/// One pixel of the tall-grass tuft: a clump of blades rooted at the bottom of
/// the tile with transparent gaps between them.
///
/// The tile is drawn on a cross of two quads, so its silhouette *is* the plant —
/// the alpha here is doing the same job the mesh does for a cube block.
fn tall_grass_pixel(x: usize, y: usize, n: f32) -> [u8; 4] {
    let h = blade_height(x);
    let from_bottom = TILE - 1 - y;
    if h == 0 || from_bottom >= h {
        return [0, 0, 0, 0];
    }
    // Tips run lighter and yellower than the roots, which reads as blades
    // catching the light rather than a flat green smear.
    let t = from_bottom as f32 / h as f32;
    let base = [
        lerp(56.0, 126.0, t) as u8,
        lerp(110.0, 170.0, t) as u8,
        lerp(38.0, 60.0, t) as u8,
    ];
    shade(base, n, 16.0)
}

/// Height in pixels of the grass blade in column `x`; 0 leaves a gap. Tallest in
/// the middle so the tuft reads as a rounded clump, with per-column jitter so the
/// top edge stays ragged.
fn blade_height(x: usize) -> usize {
    // Gaps and heights come from independent hashes: sharing one made the gaps
    // fall only among the shortest blades, which packed the tuft into a solid
    // green mass instead of separate stalks.
    if hash(x as i32 * 29, 43) < 0.34 {
        return 0;
    }
    let r = hash(x as i32 * 17, 91);
    let centre = 1.0 - ((x as f32 - 7.5).abs() / 8.0);
    (2.0 + centre * 7.0 + r * 5.0) as usize
}

/// One pixel of a flower: blossom, then stem, then leaves, checked in that order
/// so the stem never draws a green line through the middle of the blossom.
fn flower_pixel(x: usize, y: usize, n: f32, petal: [u8; 3], eye: [u8; 3]) -> [u8; 4] {
    // Blossom: a slightly squashed disc sitting at the top of the stem.
    let dx = x as f32 - 7.5;
    let dy = y as f32 - 4.5;
    let d = (dx * dx + dy * dy * 1.3).sqrt();
    if d < 1.7 {
        return shade(eye, n, 10.0);
    }
    if d < 3.8 {
        return shade(petal, n, 18.0);
    }

    let (fx, fy) = (x as i32, y as i32);
    if (fx == 7 || fx == 8) && (8..TILE as i32).contains(&fy) {
        return shade([46, 104, 40], n, 12.0); // stem
    }
    if (fy == 10 && (5..7).contains(&fx)) || (fy == 12 && (9..11).contains(&fx)) {
        return shade([54, 120, 44], n, 12.0); // leaves
    }
    [0, 0, 0, 0]
}

/// One pixel of the water tile at a given wave `phase` (radians). Advancing the
/// phase over time marches the crests across the surface; `animate_water` in
/// `water.rs` repaints the tile each frame.
///
/// Every water block's top face repeats this one tile, so the pattern must wrap:
/// the noise grids and the waves all divide TILE evenly, leaving it seamless
/// against itself in both axes no matter the phase.
///
/// Note the colour is deliberately a strong, bright blue. The surface is alpha
/// blended over the lakebed, and a bright sandy bed showing through will wash a
/// timid blue out to grey — the tile has to carry the hue on its own.
fn water_pixel(x: usize, y: usize, n: f32, phase: f32) -> [u8; 4] {
    let fx = x as f32 / TILE as f32;
    let fy = y as f32 / TILE as f32;
    let (t, chop) = water_field(fx, fy, phase);

    // Trough -> crest, aimed at the deep blue of the underwater overlay in
    // `water.rs`. Deliberately darker than it needs to look on its own: the top
    // face is the brightest in `FACE_SHADE` and takes full sun plus ambient, and
    // the tonemapper pulls anything that bright towards white. A tile that reads
    // "correct" flat here comes out bleached in game, so the hue has to sit low
    // enough to survive being lit.
    let deep = [12.0, 62.0, 145.0];
    let lit = [36.0, 112.0, 205.0];
    let base = [
        lerp(deep[0], lit[0], t) as u8,
        lerp(deep[1], lit[1], t) as u8,
        lerp(deep[2], lit[2], t) as u8,
    ];
    // Thin glints on the highest crests, where the surface catches the sky. The
    // threshold breathes with the phase so they twinkle in and out rather than
    // sliding across as a rigid pattern.
    let gate = 0.90 + 0.05 * (phase * 2.3 + chop * std::f32::consts::TAU).sin();
    if t > gate {
        shade([116, 182, 238], n, 14.0)
    } else {
        shade(base, n, 12.0)
    }
}

/// The water surface height at a tile UV and wave `phase`, as `(t, chop)`: `t`
/// runs 0 (trough) to 1 (crest), `chop` is the fine noise the glints key off.
///
/// A straight sine reads as stripes; warping its phase by wrapping noise bends
/// the crests into meandering wavefronts. Three waves cross here at different
/// frequencies, directions and speeds — the fast diagonal one is what stops the
/// motion from reading as a single image sliding by.
///
/// Everything in here must wrap over [0,1] in both axes for the tile to abut
/// itself seamlessly: `cloud_noise` wraps, and every frequency is a whole number
/// of turns. `water_seams_wrap_at_any_phase` holds this.
fn water_field(fx: f32, fy: f32, phase: f32) -> (f32, f32) {
    let tau = std::f32::consts::TAU;
    let swell = 0.65 * cloud_noise(fx, fy, 4) + 0.35 * cloud_noise(fx, fy, 8);
    let chop = cloud_noise(fx, fy, 8);
    let wave = 0.50 * ((fy * tau * 2.0) + swell * 5.0 + phase).sin()
        + 0.30 * ((fx * tau) - swell * 4.0 - phase * 0.7).sin()
        + 0.20 * (((fx + fy) * tau * 3.0) + chop * 6.0 + phase * 1.6).sin();
    (0.5 + 0.5 * wave, chop)
}

/// Repaint the atlas's water tile in place at `phase`. Only the 16x16 water tile
/// is touched; the rest of the atlas is left alone.
pub fn write_water_tile(image: &mut Image, phase: f32) {
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let w = COLS * TILE;
    let col = (T_WATER as usize) % COLS;
    let row = (T_WATER as usize) / COLS;
    for y in 0..TILE {
        for x in 0..TILE {
            // Same per-pixel grain seed as `tile_pixel`, so the static tile and
            // the animated one grain identically.
            let n = hash(T_WATER as i32 * 131 + x as i32, y as i32);
            let c = water_pixel(x, y, n, phase);
            let i = ((row * TILE + y) * w + col * TILE + x) * 4;
            data[i..i + 4].copy_from_slice(&c);
        }
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
        // Plants show the same tile whatever way you look at them.
        Block::TallGrass => T_TALL_GRASS,
        Block::RedFlower => T_FLOWER_RED,
        Block::YellowFlower => T_FLOWER_YELLOW,
        Block::RoofTile => T_ROOF_TILE,
        Block::Plaster => T_PLASTER,
        Block::Paper => T_PAPER,
        Block::Dancheong => T_DANCHEONG,
        Block::RedPillar => T_RED_PILLAR,
        Block::RoofRidge => T_ROOF_RIDGE,
        Block::Granite => T_GRANITE,
        Block::Thatch => T_THATCH,
        Block::ClayWall => T_CLAY_WALL,
        Block::Road => T_ROAD,
    }
}

#[cfg(test)]
mod preview {
    use super::*;
    use std::io::Write;

    /// The wave phase must actually move the surface, and the tile must stay
    /// seamless at any phase (opposite edges sample the same wrapped pattern).
    #[test]
    fn water_phase_animates_and_stays_seamless() {
        let at = |x: usize, y: usize, p: f32| {
            water_pixel(x, y, hash(T_WATER as i32 * 131 + x as i32, y as i32), p)
        };
        let moved = (0..TILE)
            .flat_map(|y| (0..TILE).map(move |x| (x, y)))
            .filter(|&(x, y)| at(x, y, 0.0) != at(x, y, 1.0))
            .count();
        assert!(moved > 40, "phase barely changed the tile ({moved} px)");

    }

    /// Wrapping: a UV of 0.0 and 1.0 are the same point on the torus, so the
    /// surface must agree there for the tile to abut itself without a seam.
    /// Checked through `water_field` itself, in both axes, so every wave term is
    /// covered rather than a copy of one of them.
    #[test]
    fn water_seams_wrap_at_any_phase() {
        for p in [0.0, 1.7, 4.2] {
            for k in 0..TILE {
                let a = k as f32 / TILE as f32;
                let (tx0, cx0) = water_field(0.0, a, p);
                let (tx1, cx1) = water_field(1.0, a, p);
                assert!((tx0 - tx1).abs() < 1e-4, "wave seam in x at {a}, phase {p}");
                assert!((cx0 - cx1).abs() < 1e-4, "noise seam in x at {a}, phase {p}");

                let (ty0, cy0) = water_field(a, 0.0, p);
                let (ty1, cy1) = water_field(a, 1.0, p);
                assert!((ty0 - ty1).abs() < 1e-4, "wave seam in y at {a}, phase {p}");
                assert!((cy0 - cy1).abs() < 1e-4, "noise seam in y at {a}, phase {p}");
            }
        }
    }

    /// TEMPORARY: dump a tile 3x3-tiled to a BMP so the pattern and its seams can
    /// be eyeballed. `cargo test -- --nocapture preview`.
    #[test]
    fn preview_water_tile() {
        const REP: usize = 3;
        const SCALE: usize = 8;
        let w = TILE * REP * SCALE;
        let h = w;
        let mut px = vec![0u8; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let c = tile_pixel(T_WATER, (x / SCALE) % TILE, (y / SCALE) % TILE);
                let i = (y * w + x) * 3;
                px[i] = c[2]; // BMP is BGR
                px[i + 1] = c[1];
                px[i + 2] = c[0];
            }
        }
        // 24-bit BMP, rows bottom-up. Width is chosen so rows need no padding.
        assert_eq!((w * 3) % 4, 0);
        let size = 54 + px.len();
        let out = std::env::temp_dir().join("water_preview.bmp");
        println!("preview -> {}", out.display());
        let mut f = std::fs::File::create(&out).unwrap();
        f.write_all(b"BM").unwrap();
        f.write_all(&(size as u32).to_le_bytes()).unwrap();
        f.write_all(&[0u8; 4]).unwrap();
        f.write_all(&54u32.to_le_bytes()).unwrap();
        f.write_all(&40u32.to_le_bytes()).unwrap();
        f.write_all(&(w as i32).to_le_bytes()).unwrap();
        f.write_all(&(h as i32).to_le_bytes()).unwrap();
        f.write_all(&1u16.to_le_bytes()).unwrap();
        f.write_all(&24u16.to_le_bytes()).unwrap();
        f.write_all(&[0u8; 24]).unwrap();
        for y in (0..h).rev() {
            f.write_all(&px[y * w * 3..(y + 1) * w * 3]).unwrap();
        }
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
