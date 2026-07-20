//! 조선 — the Joseon-dynasty map.
//!
//! Being rebuilt from scratch. Right now it is a bare level plain with 경복궁
//! standing on it and nothing else: no terrain relief, no rice paddies, no town,
//! no planting. Everything else gets added back one element at a time.
//!
//! The palace runs south to north along one axis, as the real one does — the
//! ceremonial buildings at the south end, the ones the court lived in behind:
//!
//! ```text
//!                        향원정   (hexagonal pavilion, on its island)
//!                        아미산   (terraced garden)
//!                        교태전   ┐ 무량각 — the king's and
//!             수정전     강녕전   ┘ queen's halls have no ridge     자경전
//!                        사정전   (the council hall)               동궁
//!                        근정전   (throne hall, on its 월대)
//!         회랑 ┌──────────────┐ 회랑     경회루 ── on its pond
//!              │  품계석 ∙ ∙  │
//!              └──── 근정문 ──┘
//!                      │  삼도 (the raised processional way)
//!                   광화문   (main gate, in the south wall)
//! ```
//!
//! Inside 근정전, at the head of that axis, stands the 어좌 under its 닫집 canopy
//! with the 일월오봉도 screen behind it.

use crate::block::Block;
use crate::world::{World, FLAT_LEVEL, WORLD_X, WORLD_Z};

/// The level everything is built on. Shared with the blank map so the two flat
/// worlds sit at the same height.
const GROUND: i32 = FLAT_LEVEL;

pub fn generate(_seed: u32) -> World {
    let mut world = World::empty();
    world.fill_flat(GROUND);
    place_palace(&mut world, GROUND);
    world
}

// --- 지붕 (the shared roof builder) ----------------------------------------

/// Lay a hipped 기와 roof over a body of half-extents `(bx, bz)` centred on
/// `(cx, cz)`, starting at `base_y`. Returns the level just above the ridge.
///
/// Three details do the work of making this read as Korean rather than as a
/// generic pyramid:
///
/// * **`step`** — how much each course draws in. Stepping in 1 per level is a
///   45° roof, which looks right on a small house and absurdly steep on a wide
///   hall. Palace buildings step 2, giving the shallow 1:2 pitch of a real
///   기와지붕. Each course fills a *band* `step` wide, not a one-block ring, or
///   a step of 2 would leave gaps you could see the sky through.
/// * **`overhang`** — the 처마. Korean eaves project far past the wall; without
///   this the roof sits on the walls like a lid.
/// * **corner lift** — a block raised at each corner of the eaves course. It is
///   the closest a voxel grid gets to the upward sweep of a 추녀, and it is the
///   silhouette people actually recognise.
///
/// `ridged` says whether the crown gets the white 양성바름 ridge. Set it false
/// for 무량각 — the ridgeless roof used over the rooms where the king and queen
/// slept. It is one block wide on the model and unmistakable in silhouette.
#[allow(clippy::too_many_arguments)]
fn lay_roof(
    world: &mut World,
    cx: i32,
    cz: i32,
    bx: i32,
    bz: i32,
    base_y: i32,
    overhang: i32,
    step: i32,
    ridged: bool,
) -> i32 {
    let (mut rx, mut rz) = (bx + overhang, bz + overhang);
    let (eave_x, eave_z) = (rx, rz);
    let mut y = base_y;

    // Draw in until *either* axis runs out. Looping on `rz` alone assumes it is
    // the shorter side; a corridor whose long axis is Z would step down its
    // length instead of its width and grow a roof dozens of courses tall.
    while rx >= 0 && rz >= 0 {
        // Whether this is the crowning course. Testing `== 0` instead looks
        // right but silently skips the ridge whenever `step` strides past zero —
        // 광화문 has an odd half-depth, so its courses ran 5, 3, 1, -1 and it
        // came out capped in plain dark tile with no 용마루 at all.
        let last = rx - step < 0 || rz - step < 0;
        let (inner_x, inner_z) = (rx - step, rz - step);
        for dz in -rz..=rz {
            for dx in -rx..=rx {
                let in_band = dx.abs() > inner_x || dz.abs() > inner_z;
                if in_band || last {
                    // The crown is the white lime-plastered 양성바름 ridge;
                    // everything below it is plain dark tile.
                    let tile = if last && ridged {
                        Block::RoofRidge
                    } else {
                        Block::RoofTile
                    };
                    world.set(cx + dx, y, cz + dz, tile);
                }
            }
        }
        y += 1;
        rx -= step;
        rz -= step;
    }

    // 추녀 — lift the four corners of the eaves. A ridgeless roof still has
    // these: it is the 용마루 that is missing, not the corner rafters.
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            world.set(cx + sx * eave_x, base_y + 1, cz + sz * eave_z, Block::RoofRidge);
        }
    }
    y
}

// --- 경복궁 (the palace) ----------------------------------------------------

/// Half-extents of the walled palace precinct. Wide enough to hold the throne
/// hall's court on the central axis *and* 경회루 on its pond off to the west,
/// the way Gyeongbokgung is actually laid out.
const PALACE_X: i32 = 38;
/// How far the precinct runs south and north of its centre. Gyeongbokgung is far
/// deeper than it is wide, and lopsided about its middle: the ceremonial gate and
/// court sit at the south end, and the halls the royal family actually lived in
/// run away north behind them.
const PALACE_SOUTH: i32 = 30;
const PALACE_NORTH: i32 = 94;
/// Palace roofs step in 2 per course. At `step` 1 a hall this wide would carry a
/// roof taller than the building; 2 gives the shallow pitch of the real thing.
const PALACE_STEP: i32 = 2;

/// Half-extents of the 근정전 court — the cloistered inner yard the throne hall
/// stands in. A throne hall alone in an open field reads as a big shed; the
/// enclosure is what makes it the centre of a palace.
const COURT_X: i32 = 20;
const COURT_Z: i32 = 15;
/// How far north of the precinct centre that court sits, leaving a long
/// approach between 광화문 and its gate.
const COURT_OFFSET_Z: i32 = -13;

/// Centres of the halls behind the throne hall, north of the court, as offsets
/// from the precinct centre. Each stands in its own walled yard.
const SAJEONG_Z: i32 = -38; // 사정전, where the king held council
const GANGNYEONG_Z: i32 = -52; // 강녕전, the king's own quarters
const GYOTAE_Z: i32 = -66; // 교태전, the queen's
/// 향원정, in the rear garden well beyond the living quarters.
const HYANGWON_Z: i32 = -84;
/// The side compounds, in the strips between the inner yards and the precinct
/// wall. Their half-width is 7, so a centre of 30 spans 23..37 — clear of both
/// the court cloister's eaves at 22 and the precinct wall at 38.
const JAGYEONG_X: i32 = 30; // 자경전, the dowager queen's hall
const JAGYEONG_Z: i32 = -52;
const SUJEONG_X: i32 = -30; // 수정전, west of the axis
const SUJEONG_Z: i32 = -36;
const DONGGUNG_X: i32 = 30; // 동궁, the crown prince's quarters
const DONGGUNG_Z: i32 = -34;

/// Half-extents of the throne hall's two 월대 terraces.
const WOLDAE_OUTER: (i32, i32) = (15, 12);
const WOLDAE_INNER: (i32, i32) = (12, 9);

/// Build 경복궁 at the centre of the map: a walled precinct entered from the
/// south through 광화문, with 근정전 raised on its 월대 terraces at the north end
/// and a stone-paved court between them.
fn place_palace(world: &mut World, gy: i32) {
    let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
    lay_courtyard(world, cx, cz, gy);
    build_wall(world, cx, cz, gy);
    place_gate(world, cx, cz + PALACE_SOUTH, gy);

    // The inner court, on the central axis: 근정문 in its south side, the ranked
    // stones down the middle, 근정전 at the head of it, and 회랑 all the way
    // round. Built in that order so the cloister's corners overwrite the ends of
    // the gate rather than the other way about.
    let court_z = cz + COURT_OFFSET_Z;
    place_rank_stones(world, cx, court_z, gy);
    place_throne_hall(world, cx, court_z - 2, gy);
    lay_cloister(world, cx, court_z, gy);
    place_inner_gate(world, cx, court_z + COURT_Z, gy);

    // 침전 — the halls the court actually lived in, running north behind the
    // throne hall, each in its own walled yard.
    place_inner_quarters(world, cx, cz, gy);

    // 경회루 — the banquet pavilion standing on its pond, west of the axis, in
    // the strip between the court's cloister and the precinct wall.
    place_gyeonghoeru(world, cx - 30, cz - 8, gy);
    // 자경전 — the dowager queen's hall, in the matching strip to the east.
    place_jagyeongjeon(world, cx + JAGYEONG_X, cz + JAGYEONG_Z, gy);
    // 수정전 and 동궁 fill the flanks either side of the inner yards, which were
    // bare ground between the cloister and the precinct wall.
    compound_wall(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, 7, 8, false);
    place_residence(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, 5, 4, true);
    compound_wall(world, cx + DONGGUNG_X, cz + DONGGUNG_Z, gy, 7, 8, false);
    place_residence(world, cx + DONGGUNG_X, cz + DONGGUNG_Z, gy, 5, 4, true);
    // 향원정 — the hexagonal pavilion in the rear garden, at the far north.
    place_hyangwonjeong(world, cx, cz + HYANGWON_Z, gy);
}

/// A walled compound around a hall, with a gateway in its south face. Set
/// `flowered` to paint that face's lower course, as 자경전's 꽃담 is.
fn compound_wall(
    world: &mut World,
    cx: i32,
    cz: i32,
    gy: i32,
    rx: i32,
    rz: i32,
    flowered: bool,
) {
    for dz in -rz..=rz {
        for dx in -rx..=rx {
            if dx.abs() != rx && dz.abs() != rz {
                continue;
            }
            if dz == rz && dx.abs() <= 1 {
                continue; // gateway
            }
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            let painted = flowered && dz == rz;
            world.set(
                cx + dx,
                gy + 2,
                cz + dz,
                if painted {
                    Block::Dancheong
                } else {
                    Block::Plaster
                },
            );
            world.set(cx + dx, gy + 3, cz + dz, Block::Plaster);
            world.set(cx + dx, gy + 4, cz + dz, Block::RoofTile);
        }
    }
}

/// Board the inside of a hall out in timber. Without this you step through the
/// doors onto bare foundation stone and the building reads as a shell rather
/// than a room.
fn lay_hall_floor(world: &mut World, cx: i32, cz: i32, bx: i32, bz: i32, y: i32) {
    for dz in -(bz - 1)..=(bz - 1) {
        for dx in -(bx - 1)..=(bx - 1) {
            world.set(cx + dx, y, cz + dz, Block::Wood);
        }
    }
}

// --- 자경전 (the dowager queen's hall) ---------------------------------------

/// 자경전 in its own walled compound, with the 꽃담 — the patterned wall the
/// real one is known for — along its south side.
fn place_jagyeongjeon(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // 꽃담 — the patterned wall this hall is known for. Only its south face gets
    // the painted course; it is the side you see, and decorating all four would
    // spend the effect.
    compound_wall(world, cx, cz, gy, 7, 11, true);
    place_residence(world, cx, cz, gy, 5, 4, true);

    // 십장생 굴뚝 — the tall decorated chimney standing in the yard behind.
    let chimney_z = cz - 9;
    for h in 1..=6 {
        world.set(cx, gy + h, chimney_z, Block::ClayWall);
    }
    world.set(cx, gy + 7, chimney_z, Block::Dancheong);
    world.set(cx, gy + 8, chimney_z, Block::RoofTile);
}

// --- 향원정 (the pavilion in the rear garden) --------------------------------

/// Is `(dx, dz)` inside a hexagon of radius `r`? Drawing the pavilion round
/// would waste the one chance this palace has to show a shape that isn't a
/// rectangle — 향원정 is famously six-sided.
fn in_hex(dx: i32, dz: i32, r: i32) -> bool {
    dz.abs() <= r && dx.abs() <= r - dz.abs() / 2
}

/// A stepped roof that keeps the hexagon. Running the ordinary rectangular
/// roof over 향원정 hid the six-sided plan completely — from above, the only
/// angle the shape really shows from, it came out square like everything else.
///
/// Each course draws in by one, so consecutive bands are adjacent in plan and
/// tile without gaps, and the last is the single-block finial (절병통).
fn lay_hex_roof(world: &mut World, cx: i32, cz: i32, r: i32, base_y: i32) {
    let mut rr = r + 1; // the eaves overhang by one
    let mut y = base_y;
    while rr >= 0 {
        for dz in -rr..=rr {
            for dx in -rr..=rr {
                if !in_hex(dx, dz, rr) {
                    continue;
                }
                if in_hex(dx, dz, rr - 1) && rr > 0 {
                    continue; // covered by the course above
                }
                let tile = if rr == 0 {
                    Block::RoofRidge
                } else {
                    Block::RoofTile
                };
                world.set(cx + dx, y, cz + dz, tile);
            }
        }
        y += 1;
        rr -= 1;
    }
}

/// 향원정 — a hexagonal pavilion on an island in 향원지, reached by a bridge.
fn place_hyangwonjeong(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const POND_R: i32 = 9;
    const ISLAND_R: i32 = 4;
    const HEX_R: i32 = 3;
    const DEPTH: i32 = 2;

    // The pond: a rounded basin with a dressed stone rim.
    for dz in -POND_R..=POND_R {
        for dx in -POND_R..=POND_R {
            let d2 = dx * dx + dz * dz;
            if d2 > POND_R * POND_R {
                continue;
            }
            let (x, z) = (cx + dx, cz + dz);
            if d2 > (POND_R - 1) * (POND_R - 1) {
                world.set(x, gy, z, Block::Granite); // the rim
                continue;
            }
            for d in 0..=DEPTH {
                world.set(x, gy - d, z, Block::Water);
            }
            world.set(x, gy - DEPTH - 1, z, Block::Dirt);
        }
    }

    // The island, standing back out of the water in the middle.
    for dz in -ISLAND_R..=ISLAND_R {
        for dx in -ISLAND_R..=ISLAND_R {
            if dx * dx + dz * dz > ISLAND_R * ISLAND_R {
                continue;
            }
            for d in -DEPTH..=0 {
                world.set(cx + dx, gy + d, cz + dz, Block::Granite);
            }
            world.set(cx + dx, gy + 1, cz + dz, Block::Grass);
            for h in 2..=10 {
                world.set(cx + dx, gy + h, cz + dz, Block::Air);
            }
        }
    }

    // The pavilion: a hexagonal stone floor, a column at each of its six
    // corners, and a roof.
    let floor = gy + 2;
    for dz in -HEX_R..=HEX_R {
        for dx in -HEX_R..=HEX_R {
            if in_hex(dx, dz, HEX_R) {
                world.set(cx + dx, floor, cz + dz, Block::Wood);
            }
        }
    }
    // A column wherever the hexagon's edge turns — its six corners.
    for h in 1..=3 {
        for dz in -HEX_R..=HEX_R {
            for dx in -HEX_R..=HEX_R {
                let edge = in_hex(dx, dz, HEX_R) && !in_hex(dx, dz, HEX_R - 1);
                if edge && (dz.abs() == HEX_R || dx.abs() == HEX_R - dz.abs() / 2) {
                    world.set(cx + dx, floor + h, cz + dz, Block::RedPillar);
                }
            }
        }
    }
    let beam = floor + 4;
    for dz in -HEX_R..=HEX_R {
        for dx in -HEX_R..=HEX_R {
            if in_hex(dx, dz, HEX_R) && !in_hex(dx, dz, HEX_R - 1) {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    lay_hex_roof(world, cx, cz, HEX_R, beam + 1);

    // 취향교 — the bridge out to the island, running south to the bank.
    for dz in ISLAND_R..=POND_R {
        for dx in -1..=1 {
            world.set(cx + dx, gy + 1, cz + dz, Block::Wood);
            for h in 2..=5 {
                world.set(cx + dx, gy + h, cz + dz, Block::Air);
            }
        }
    }
}

/// 난간 — balustrades round both 월대 terraces, open on the axis where the
/// stairs come up. Besides being what the real terraces have, the openings turn
/// a platform you could scramble onto anywhere into one you approach the way you
/// are meant to: up the middle, facing the throne.
fn place_terrace_rails(world: &mut World, cx: i32, cz: i32, gy: i32) {
    for (level, (rx, rz)) in [(gy + 2, WOLDAE_OUTER), (gy + 3, WOLDAE_INNER)] {
        for dz in -rz..=rz {
            for dx in -rx..=rx {
                if dx.abs() != rx && dz.abs() != rz {
                    continue;
                }
                // The stair up the south face.
                if dz == rz && dx.abs() <= 3 {
                    continue;
                }
                world.set(cx + dx, level, cz + dz, Block::Granite);
            }
        }
    }
}

// --- 침전 (the residential halls) -------------------------------------------

/// Half-width of the yards behind the throne hall. Kept clear of the strip along
/// the east wall, where 자경전 stands.
const INNER_X: i32 = 20;

/// Lay out the sequence of halls north of the throne hall: 사정전 where the king
/// held council, then 강녕전 and 교태전 where he and the queen slept, each behind
/// its own cross wall.
fn place_inner_quarters(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // 사정전 keeps a ridge; it is a hall of state like the ones to the south.
    cross_wall(world, cx, cz + SAJEONG_Z + 12, gy);
    place_residence(world, cx, cz + SAJEONG_Z, gy, 7, 4, true);

    // 강녕전 and 교태전 are 무량각 — built deliberately *without* a ridge beam
    // over the rooms where the king and queen slept.
    cross_wall(world, cx, cz + GANGNYEONG_Z + 8, gy);
    place_residence(world, cx, cz + GANGNYEONG_Z, gy, 8, 4, false);

    cross_wall(world, cx, cz + GYOTAE_Z + 8, gy);
    place_residence(world, cx, cz + GYOTAE_Z, gy, 7, 4, false);

    // 아미산 — the terraced garden behind the queen's hall. Each step is both
    // further north and one course higher, so the ground climbs away from the
    // hall towards the back wall rather than towards it.
    for step in 0..4 {
        let dz = GYOTAE_Z - 4 - step;
        for dx in -12..=12 {
            for h in 0..=step {
                world.set(cx + dx, gy + 1 + h, cz + dz, Block::Granite);
            }
        }
    }
}

/// A cross wall dividing one yard from the next, with a gateway on the axis.
fn cross_wall(world: &mut World, cx: i32, cz: i32, gy: i32) {
    for dx in -INNER_X..=INNER_X {
        if dx.abs() <= 1 {
            // The gateway — leave it open, but carry the coping across so the
            // wall reads as continuous.
            world.set(cx + dx, gy + 4, cz, Block::RoofTile);
            continue;
        }
        world.set(cx + dx, gy + 1, cz, Block::Granite);
        world.set(cx + dx, gy + 2, cz, Block::Plaster);
        world.set(cx + dx, gy + 3, cz, Block::Plaster);
        world.set(cx + dx, gy + 4, cz, Block::RoofTile);
    }
}

/// One hall of the residential quarter: a granite platform, a red-pillared body
/// with plaster walls and paper doors across the south front, a painted beam and
/// a tiled roof.
fn place_residence(
    world: &mut World,
    cx: i32,
    cz: i32,
    gy: i32,
    bx: i32,
    bz: i32,
    ridged: bool,
) {
    const BODY_H: i32 = 4;

    // 기단 — the platform, projecting a little past the walls all round.
    for dz in -(bz + 2)..=(bz + 2) {
        for dx in -(bx + 2)..=(bx + 2) {
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            for h in 2..=(BODY_H + 8) {
                world.set(cx + dx, gy + h, cz + dz, Block::Air);
            }
        }
    }

    let floor = gy + 2;
    lay_hall_floor(world, cx, cz, bx, bz, floor - 1);
    for h in 0..BODY_H {
        let y = floor + h;
        for dz in -bz..=bz {
            for dx in -bx..=bx {
                if dx.abs() != bx && dz.abs() != bz {
                    continue; // interior stays open
                }
                let corner = dx.abs() == bx && dz.abs() == bz;
                let post = corner
                    || (dx.rem_euclid(3) == 0 && dz.abs() == bz)
                    || (dz.rem_euclid(3) == 0 && dx.abs() == bx);
                let block = if post {
                    Block::RedPillar
                } else if dz == bz {
                    Block::Paper // the south front is doors
                } else {
                    Block::Plaster
                };
                world.set(cx + dx, y, cz + dz, block);
            }
        }
    }
    // A doorway through the front.
    for h in 0..3 {
        for dx in -1..=1 {
            world.set(cx + dx, floor + h, cz + bz, Block::Air);
        }
    }

    let beam = floor + BODY_H;
    for dz in -bz..=bz {
        for dx in -bx..=bx {
            if dx.abs() == bx || dz.abs() == bz {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    lay_roof(world, cx, cz, bx, bz, beam + 1, 2, PALACE_STEP, ridged);

    // 드므 — the bronze vats that stood at a hall's corners, kept full of water
    // as a charm against fire.
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (x, z) = (cx + sx * (bx + 2), cz + sz * (bz + 2));
            world.set(x, gy + 2, z, Block::Granite);
            world.set(x, gy + 3, z, Block::Water);
        }
    }
}

// --- 회랑 (the cloister) ----------------------------------------------------

/// Height of the cloister's colonnade, from its raised floor to the beam.
const CLOISTER_H: i32 = 3;

/// Run 회랑 around all four sides of the court.
fn lay_cloister(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // North and south runs, along X; then east and west, along Z. `inward`
    // points at the court, so each run knows which of its two faces is the open
    // colonnade and which is the solid outer wall.
    for side in [-1, 1] {
        cloister_run(world, cx, cz + side * COURT_Z, COURT_X, true, -side, gy);
        cloister_run(world, cx + side * COURT_X, cz, COURT_Z, false, -side, gy);
    }
}

/// One straight run of cloister: a raised walkway, solid on the outside, open
/// colonnade on the court side, under a tiled roof.
fn cloister_run(
    world: &mut World,
    cx: i32,
    cz: i32,
    half_len: i32,
    along_x: bool,
    inward: i32,
    gy: i32,
) {
    let at = |t: i32, w: i32| {
        if along_x {
            (cx + t, cz + w)
        } else {
            (cx + w, cz + t)
        }
    };

    for t in -half_len..=half_len {
        // A raised granite walkway, three wide, centred on the court boundary.
        for w in -1..=1 {
            let (x, z) = at(t, w);
            world.set(x, gy + 1, z, Block::Granite);
            for h in 2..=(CLOISTER_H + 5) {
                world.set(x, gy + h, z, Block::Air);
            }
        }

        // Posts every third bay. The outer face is walled between them; the
        // court face is left open, which is what makes it a colonnade and not a
        // corridor you cannot see out of.
        let post = t.rem_euclid(3) == 0;
        for h in 2..=(CLOISTER_H + 1) {
            let (ox, oz) = at(t, -inward);
            world.set(
                ox,
                gy + h,
                oz,
                if post { Block::RedPillar } else { Block::Plaster },
            );
            if post {
                let (ix, iz) = at(t, inward);
                world.set(ix, gy + h, iz, Block::RedPillar);
            }
        }

        // Painted beam over both faces.
        let beam = gy + CLOISTER_H + 2;
        for w in [-1, 1] {
            let (x, z) = at(t, w);
            world.set(x, beam, z, Block::Dancheong);
        }
    }

    // One roof over the whole run. Half-extents are in world space, so they swap
    // with the run's direction.
    let (bx, bz) = if along_x { (half_len, 1) } else { (1, half_len) };
    lay_roof(world, cx, cz, bx, bz, gy + CLOISTER_H + 3, 1, 1, true);
}

/// 근정문 — the inner gate in the south side of the court, on the axis between
/// 광화문 and the throne hall.
fn place_inner_gate(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const GX: i32 = 7;
    const GZ: i32 = 2;
    const BODY_H: i32 = 4;

    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            for h in 2..=(BODY_H + 6) {
                world.set(cx + dx, gy + h, cz + dz, Block::Air);
            }
        }
    }

    let floor = gy + 2;
    for h in 0..BODY_H {
        let y = floor + h;
        for dz in -GZ..=GZ {
            for dx in -GX..=GX {
                if dx.abs() != GX && dz.abs() != GZ {
                    continue; // the passage through the middle stays open
                }
                // Three doorways in each face: the king's in the centre, an
                // officials' door either side.
                let doorway = dz.abs() == GZ && h < 3 && (dx.abs() <= 1 || (4..=5).contains(&dx.abs()));
                if doorway {
                    world.set(cx + dx, y, cz + dz, Block::Air);
                    continue;
                }
                let post = dx.rem_euclid(3) == 0 || (dx.abs() == GX && dz.abs() == GZ);
                world.set(
                    cx + dx,
                    y,
                    cz + dz,
                    if post { Block::RedPillar } else { Block::Paper },
                );
            }
        }
    }

    let beam = floor + BODY_H;
    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            if dx.abs() == GX || dz.abs() == GZ {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    lay_roof(world, cx, cz, GX, GZ, beam + 1, 2, PALACE_STEP, true);
}

/// 품계석 — the ranked stones officials lined up beside, in two rows down the
/// court flanking the 삼도.
fn place_rank_stones(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let mut z = cz + COURT_Z - 5;
    while z > cz - COURT_Z + 8 {
        for dx in [-5, 5] {
            world.set(cx + dx, gy + 1, z, Block::Granite);
        }
        z -= 3;
    }
}

// --- 경회루 (the pavilion on the pond) --------------------------------------

/// Dig a pond and stand 경회루 in the middle of it on stone pillars, with a
/// causeway back to the bank.
///
/// The pavilion has no walls at all — it is a roof on columns, open on every
/// side, which is exactly what it was for.
fn place_gyeonghoeru(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // The pond has to fit the strip between the court's west cloister and the
    // precinct wall — about sixteen blocks — so it is long north-to-south rather
    // than square.
    const POND_X: i32 = 7;
    const POND_Z: i32 = 12;
    const DEPTH: i32 = 2;
    /// Half-extents of the pavilion's stone understructure.
    const BASE_X: i32 = 4;
    const BASE_Z: i32 = 4;

    // Excavate, then flood to a hair below the bank so the water reads as a
    // pond rather than as a flooded courtyard.
    for dz in -POND_Z..=POND_Z {
        for dx in -POND_X..=POND_X {
            let (x, z) = (cx + dx, cz + dz);
            if dx.abs() == POND_X || dz.abs() == POND_Z {
                world.set(x, gy, z, Block::Granite); // dressed stone bank
                continue;
            }
            for d in 0..=DEPTH {
                world.set(x, gy - d, z, Block::Water);
            }
            world.set(x, gy - DEPTH - 1, z, Block::Dirt);
        }
    }

    // The stone forest the pavilion stands on: square columns rising out of the
    // water on a regular grid.
    for dz in -BASE_Z..=BASE_Z {
        for dx in -BASE_X..=BASE_X {
            let pillar = dx.rem_euclid(2) == 0 && dz.rem_euclid(2) == 0;
            if pillar {
                for d in -DEPTH..=2 {
                    world.set(cx + dx, gy + d, cz + dz, Block::Granite);
                }
            }
        }
    }

    // Timber deck across the top of them.
    let floor = gy + 3;
    for dz in -BASE_Z..=BASE_Z {
        for dx in -BASE_X..=BASE_X {
            world.set(cx + dx, floor, cz + dz, Block::Wood);
            for h in 1..=8 {
                world.set(cx + dx, floor + h, cz + dz, Block::Air);
            }
        }
    }

    // Open colonnade: columns only, no infill.
    for h in 1..=3 {
        for dz in -BASE_Z..=BASE_Z {
            for dx in -BASE_X..=BASE_X {
                let edge = dx.abs() == BASE_X || dz.abs() == BASE_Z;
                if edge && dx.rem_euclid(2) == 0 && dz.rem_euclid(2) == 0 {
                    world.set(cx + dx, floor + h, cz + dz, Block::RedPillar);
                }
            }
        }
    }
    let beam = floor + 4;
    for dz in -BASE_Z..=BASE_Z {
        for dx in -BASE_X..=BASE_X {
            if dx.abs() == BASE_X || dz.abs() == BASE_Z {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    lay_roof(world, cx, cz, BASE_X, BASE_Z, beam + 1, 2, PALACE_STEP, true);

    // A causeway east to the bank, at deck height. It starts *beyond* the
    // pavilion's own edge: running it from `BASE_X` cleared the colonnade and
    // deck it was supposed to join, leaving the pavilion open on that side.
    for dx in (BASE_X + 1)..=POND_X {
        world.set(cx + dx, floor, cz, Block::Granite);
        for h in 1..=4 {
            world.set(cx + dx, floor + h, cz, Block::Air);
        }
    }
}

/// Pave the ceremonial half of the precinct in granite and run the 삼도 — the
/// raised processional way — from 광화문 up to the throne hall.
///
/// Only the southern, ceremonial half is paved. The residential yards behind the
/// throne hall keep the bare ground, with each hall standing on its own stone
/// platform; paving the whole precinct made it read as one enormous parade
/// ground rather than a sequence of separate courts.
fn lay_courtyard(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let paved_north = COURT_OFFSET_Z - COURT_Z - 2;
    for dz in paved_north..=PALACE_SOUTH {
        for dx in -PALACE_X..=PALACE_X {
            world.set(cx + dx, gy, cz + dz, Block::Granite);
        }
    }
    for dz in (COURT_OFFSET_Z - 2)..=PALACE_SOUTH {
        for dx in -3..=3 {
            // The centre lane sits a block proud of the two flanking it.
            let block = if dx == 0 { Block::Granite } else { Block::Stone };
            world.set(cx + dx, gy, cz + dz, block);
            if dx == 0 {
                world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            }
        }
    }
}

/// The 담장 around the precinct: granite footing, plaster body, tiled coping —
/// left open where 광화문 stands.
fn build_wall(world: &mut World, cx: i32, cz: i32, gy: i32) {
    for dz in -PALACE_NORTH..=PALACE_SOUTH {
        for dx in -PALACE_X..=PALACE_X {
            let on_side = dx.abs() == PALACE_X;
            let on_end = dz == PALACE_SOUTH || dz == -PALACE_NORTH;
            if !on_side && !on_end {
                continue;
            }
            // Leave the gateway clear.
            if dz == PALACE_SOUTH && dx.abs() <= 7 {
                continue;
            }
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            world.set(cx + dx, gy + 2, cz + dz, Block::Plaster);
            world.set(cx + dx, gy + 3, cz + dz, Block::Plaster);
            world.set(cx + dx, gy + 4, cz + dz, Block::RoofTile);
        }
    }
}

/// 광화문 — the main gate: a granite base pierced by three arched passages,
/// carrying a painted timber storey and a tiled roof.
fn place_gate(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const GX: i32 = 8; // half-width
    const GZ: i32 = 3; // half-depth
    const BASE_H: i32 = 5;

    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            for h in 1..=BASE_H {
                world.set(cx + dx, gy + h, cz + dz, Block::Granite);
            }
        }
    }

    // Three passages through the base. The middle one — the king's — is taller.
    for (centre, height) in [(-5i32, 3i32), (0, 4), (5, 3)] {
        for dz in -GZ..=GZ {
            for dx in -1i32..=1 {
                for h in 1..=height {
                    // Round the top off so the opening reads as an arch.
                    if h == height && dx.abs() == 1 {
                        continue;
                    }
                    world.set(cx + centre + dx, gy + h, cz + dz, Block::Air);
                }
            }
        }
    }

    // The painted storey above: red columns, dancheong beams, paper infill.
    let floor = gy + BASE_H + 1;
    for h in 0..3 {
        let y = floor + h;
        for dz in -GZ..=GZ {
            for dx in -GX..=GX {
                if dx.abs() != GX && dz.abs() != GZ {
                    continue;
                }
                let post = dx.rem_euclid(4) == 0 || dz.abs() == GZ && dx.abs() == GX;
                world.set(
                    cx + dx,
                    y,
                    cz + dz,
                    if post { Block::RedPillar } else { Block::Paper },
                );
            }
        }
    }
    let beam = floor + 3;
    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            if dx.abs() == GX || dz.abs() == GZ {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    lay_roof(world, cx, cz, GX, GZ, beam + 1, 2, PALACE_STEP, true);
}

/// 근정전 — the throne hall, on two granite 월대 terraces, with the double roof
/// that gives it its silhouette.
fn place_throne_hall(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // 월대 — two stepped terraces, the lower broad enough to walk around.
    terrace(world, cx, cz, WOLDAE_OUTER.0, WOLDAE_OUTER.1, gy + 1);
    terrace(world, cx, cz, WOLDAE_INNER.0, WOLDAE_INNER.1, gy + 2);
    place_terrace_rails(world, cx, cz, gy);

    let floor = gy + 3;
    // Lower storey: a colonnade of red pillars with plaster and paper between.
    hall_storey(world, cx, cz, 9, 6, floor, 4);
    lay_hall_floor(world, cx, cz, 9, 6, floor - 1);
    place_throne(world, cx, cz - 2, floor);
    let lower_beam = floor + 4;
    lay_roof(world, cx, cz, 9, 6, lower_beam + 1, 2, PALACE_STEP, true);

    // Upper storey rising through the lower roof — the 중층 that makes 근정전
    // read as a throne hall rather than a large shed. It starts above the lower
    // roof's first two courses so it emerges from them instead of being buried.
    let upper_floor = lower_beam + 4;
    hall_storey(world, cx, cz, 6, 4, upper_floor, 3);
    let upper_beam = upper_floor + 3;
    lay_roof(world, cx, cz, 6, 4, upper_beam + 1, 2, PALACE_STEP, true);
}

/// 어좌 — the throne, on its dais at the north end of the hall, under a 닫집
/// canopy and in front of the 일월오봉도 screen.
///
/// The hall was an empty shell until now: you could walk in through 광화문, up
/// the 삼도, through 근정문 and into the building, and find nothing at all. This
/// is what the whole axis points at.
fn place_throne(world: &mut World, cx: i32, cz: i32, floor: i32) {
    // The dais, stepping up twice.
    for dz in -2..=1 {
        for dx in -3..=3 {
            world.set(cx + dx, floor, cz + dz, Block::Granite);
        }
    }
    for dz in -2..=0 {
        for dx in -2..=2 {
            world.set(cx + dx, floor + 1, cz + dz, Block::Granite);
        }
    }
    world.set(cx, floor + 2, cz - 1, Block::RedPillar); // the seat

    // 일월오봉도 — the sun, moon and five peaks, which stood behind the throne
    // wherever the king sat. The painting itself is far below this resolution;
    // what carries is a band of colour filling the wall right behind the seat.
    for dx in -3..=3 {
        for h in 2..=4 {
            world.set(cx + dx, floor + h, cz - 3, Block::Dancheong);
        }
    }

    // 닫집 — the canopy, on four posts over the seat.
    for sx in [-2, 2] {
        for sz in [-2, 1] {
            for h in 3..=4 {
                world.set(cx + sx, floor + h, cz + sz, Block::RedPillar);
            }
        }
    }
    for dz in -2..=1 {
        for dx in -2..=2 {
            world.set(cx + dx, floor + 5, cz + dz, Block::RoofTile);
        }
    }
    for dx in -1..=1 {
        world.set(cx + dx, floor + 6, cz, Block::RoofRidge);
    }
}

fn terrace(world: &mut World, cx: i32, cz: i32, rx: i32, rz: i32, y: i32) {
    for dz in -rz..=rz {
        for dx in -rx..=rx {
            world.set(cx + dx, y, cz + dz, Block::Granite);
        }
    }
}

/// One storey of a palace hall: red columns on a regular bay spacing, wall
/// infill between them, and a painted beam capping it.
fn hall_storey(world: &mut World, cx: i32, cz: i32, bx: i32, bz: i32, floor: i32, height: i32) {
    for h in 0..height {
        let y = floor + h;
        for dz in -bz..=bz {
            for dx in -bx..=bx {
                if dx.abs() != bx && dz.abs() != bz {
                    continue;
                }
                // Columns every third bay, and at every corner.
                let corner = dx.abs() == bx && dz.abs() == bz;
                let bay = dx.rem_euclid(3) == 0 && dz.abs() == bz
                    || dz.rem_euclid(3) == 0 && dx.abs() == bx;
                let block = if corner || bay {
                    Block::RedPillar
                } else if dz == bz {
                    Block::Paper // the south front is doors
                } else {
                    Block::Plaster
                };
                world.set(cx + dx, y, cz + dz, block);
            }
        }
    }
    // Doorway through the front.
    for h in 0..3.min(height) {
        for dx in -1..=1 {
            world.set(cx + dx, floor + h, cz + bz, Block::Air);
        }
    }
    let beam = floor + height;
    for dz in -bz..=bz {
        for dx in -bx..=bx {
            if dx.abs() == bx || dz.abs() == bz {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    use crate::world::WORLD_Y;

    /// 무량각 — the king's and queen's halls were built deliberately *without* a
    /// ridge beam, while 사정전 immediately south of them keeps one. It is a
    /// one-block difference that a change to the roof builder could silently
    /// erase, and it is the whole reason `lay_roof` takes a `ridged` flag.
    #[test]
    fn sleeping_halls_have_no_ridge() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        let crown = |dz: i32| {
            let z = cz + dz;
            let y = (0..WORLD_Y)
                .rev()
                .find(|&y| w.get(cx, y, z) != Block::Air)
                .expect("nothing was built on the axis here");
            w.get(cx, y, z)
        };
        assert_eq!(crown(SAJEONG_Z), Block::RoofRidge, "사정전 lost its ridge");
        assert_eq!(crown(GANGNYEONG_Z), Block::RoofTile, "강녕전 should be 무량각");
        assert_eq!(crown(GYOTAE_Z), Block::RoofTile, "교태전 should be 무량각");
    }

    /// Every hall the palace is meant to contain has to actually be standing.
    /// Buildings placed near one another have silently razed each other twice in
    /// this map's history — the town levelled 광화문, and 경회루's causeway ate
    /// its own colonnade — and a hall that got overwritten looks exactly like one
    /// that was never added.
    #[test]
    fn every_hall_is_standing() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        for (name, x, z) in [
            ("사정전", cx, cz + SAJEONG_Z),
            ("강녕전", cx, cz + GANGNYEONG_Z),
            ("교태전", cx, cz + GYOTAE_Z),
            ("자경전", cx + JAGYEONG_X, cz + JAGYEONG_Z),
            ("수정전", cx + SUJEONG_X, cz + SUJEONG_Z),
            ("동궁", cx + DONGGUNG_X, cz + DONGGUNG_Z),
            ("향원정", cx, cz + HYANGWON_Z),
        ] {
            let columns = count_in(&w, x, z, 8, 8, Block::RedPillar);
            let painted = count_in(&w, x, z, 8, 8, Block::Dancheong);
            assert!(columns > 8, "{name} has no columns ({columns})");
            assert!(painted > 8, "{name} has no painted beam ({painted})");
        }
    }

    /// The whole axis — gate, processional way, inner gate, terraces — points at
    /// the throne. It is the one thing in the palace you have to go inside to
    /// see, so nothing else would reveal it going missing.
    #[test]
    fn the_throne_hall_has_a_throne() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        let hz = cz + COURT_OFFSET_Z - 2;
        let floor = GROUND + 3;
        assert_eq!(
            w.get(cx, floor + 2, hz - 3),
            Block::RedPillar,
            "the throne seat is missing"
        );
        // 일월오봉도 stands behind it, and the 닫집 hangs over it.
        assert_eq!(
            w.get(cx, floor + 3, hz - 5),
            Block::Dancheong,
            "the 일월오봉도 screen is missing"
        );
        assert_eq!(
            w.get(cx, floor + 6, hz - 2),
            Block::RoofRidge,
            "the 닫집 canopy is missing"
        );
    }

    /// The things that make the palace legible must be there, and you have to be
    /// able to walk in through the gate.
    #[test]
    fn palace_is_solid_and_enterable() {
        let w = generate(1337);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);

        // The centre passage of 광화문 is open all the way through.
        for dz in -3..=3 {
            for h in 1..=3 {
                let b = w.get(cx, GROUND + h, cz + PALACE_SOUTH + dz);
                assert_eq!(b, Block::Air, "gate passage blocked at dz={dz} h={h}: {b:?}");
            }
        }

        // Count inside the precinct only, so this measures the palace and not
        // whatever else the map happens to build elsewhere.
        for b in [
            Block::Dancheong,
            Block::RedPillar,
            Block::RoofRidge,
            Block::Granite,
        ] {
            let n = count_in(&w, cx, cz, PALACE_X + 6, PALACE_SOUTH + 6, b);
            println!("palace {b:?} = {n}");
            assert!(n > 30, "{b:?} barely appears in the palace ({n})");
        }
    }

    /// Nothing is planted on this map. It is easy for a `decorate` call or a
    /// planting pass to creep back in, and the whole point is that the
    /// architecture stands unobstructed.
    #[test]
    fn nothing_is_planted() {
        let w = generate(1337);
        let (mut leaves, mut plants) = (0u32, 0u32);
        for z in 0..WORLD_Z {
            for x in 0..WORLD_X {
                for y in 0..WORLD_Y {
                    let b = w.get(x, y, z);
                    if b == Block::Leaves {
                        leaves += 1;
                    }
                    if b.is_plant() {
                        plants += 1;
                    }
                }
            }
        }
        assert_eq!(leaves, 0, "trees were planted on the Joseon map");
        assert_eq!(plants, 0, "grass or flowers were scattered on the Joseon map");
    }

    fn count_in(w: &World, cx: i32, cz: i32, rx: i32, rz: i32, block: Block) -> u32 {
        let mut n = 0;
        for dz in -rz..=rz {
            for dx in -rx..=rx {
                for y in 0..WORLD_Y {
                    if w.get(cx + dx, y, cz + dz) == block {
                        n += 1;
                    }
                }
            }
        }
        n
    }
}
