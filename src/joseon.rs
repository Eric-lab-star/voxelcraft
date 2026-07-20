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
//!                        신무문   (north gate)
//!                        향원정   (hexagonal pavilion, on its island)
//!                        아미산   (terraced garden)
//!                        교태전   ┐ 무량각 — the king's and
//!    영추문   수정전     강녕전   ┘ queen's halls have no ridge     자경전
//!                                                                  건춘문
//!                        사정전   (the council hall)               동궁
//!                        근정전   (throne hall, on its 월대)
//!         회랑 ┌──────────────┐ 회랑     경회루 ── on its pond
//!              │  품계석 ∙ ∙  │
//!              └──── 근정문 ──┘
//!                      │  삼도 (the raised processional way)
//!                    영제교   (over 금천, the stream across the approach)
//!                    흥례문
//!                    광화문   (main gate, in the south wall)
//! ```
//!
//! Inside 근정전, at the head of that axis, stands the 어좌 under its 닫집 canopy
//! with the 일월오봉도 screen behind it.

use crate::block::Block;
use crate::world::{World, FLAT_LEVEL, WORLD_X, WORLD_Z};

/// The level everything is built on. Shared with the blank map so the two flat
/// worlds sit at the same height.
const GROUND: i32 = FLAT_LEVEL;

/// Scale every dimension in this map by 3/2.
///
/// The palace was first laid out at a scale where 근정전's body was 19 blocks
/// across. That is close to the real hall's 30m, but it left the *details* with
/// nowhere to go: a 공포 bracket set, the sweep of a 처마, the frame around a
/// 창호 panel are all sub-block at that size, so each collapsed to a single
/// block or vanished. Half again as big is the smallest step that gives them
/// two or three blocks to work in.
///
/// Dimensions are still written at the original scale and passed through here,
/// rather than being restated as scaled literals, so the relationships the
/// layout depends on stay legible — 자경전's half-width of 7 against the
/// cloister at 22 and the wall at 38 — and one edit rescales the whole map.
///
/// Deliberately *not* applied to: `PALACE_STEP`, which is a ratio rather than a
/// length, so the roofs keep their 1:2 pitch and simply grow taller with the
/// halls they cover; and the 월대 terrace courses, which are one block each and
/// have no half-block to grow by.
const fn s(n: i32) -> i32 {
    n * 3 / 2
}

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
const PALACE_X: i32 = s(38);
/// How far the precinct runs south and north of its centre. Gyeongbokgung is far
/// deeper than it is wide, and lopsided about its middle: the ceremonial gate and
/// court sit at the south end, and the halls the royal family actually lived in
/// run away north behind them.
const PALACE_SOUTH: i32 = s(30);
/// The north wall used to stand two blocks off 향원정's pond, which left no
/// room at all for 신무문 in it. Carried further out instead of squeezing the
/// pond: the real palace has a whole quarter up here, and this is where it
/// would go.
const PALACE_NORTH: i32 = s(100);
/// Palace roofs step in 2 per course. At `step` 1 a hall this wide would carry a
/// roof taller than the building; 2 gives the shallow pitch of the real thing.
const PALACE_STEP: i32 = 2;

/// Half-extents of the 근정전 court — the cloistered inner yard the throne hall
/// stands in. A throne hall alone in an open field reads as a big shed; the
/// enclosure is what makes it the centre of a palace.
const COURT_X: i32 = s(20);
const COURT_Z: i32 = s(15);
/// How far north of the precinct centre that court sits, leaving a long
/// approach between 광화문 and its gate.
const COURT_OFFSET_Z: i32 = -s(13);

/// Centres of the halls behind the throne hall, north of the court, as offsets
/// from the precinct centre. Each stands in its own walled yard.
const SAJEONG_Z: i32 = -s(38); // 사정전, where the king held council
const GANGNYEONG_Z: i32 = -s(52); // 강녕전, the king's own quarters
const GYOTAE_Z: i32 = -s(66); // 교태전, the queen's
/// 향원정, in the rear garden well beyond the living quarters.
const HYANGWON_Z: i32 = -s(84);
/// The side compounds, in the strips between the inner yards and the precinct
/// wall. Their half-width is 7, so a centre of 30 spans 23..37 — clear of both
/// the court cloister's eaves at 22 and the precinct wall at 38.
const JAGYEONG_X: i32 = s(30); // 자경전, the dowager queen's hall
const JAGYEONG_Z: i32 = -s(52);
const SUJEONG_X: i32 = -s(30); // 수정전, west of the axis
const SUJEONG_Z: i32 = -s(36);
const DONGGUNG_X: i32 = s(30); // 동궁, the crown prince's quarters
/// 동궁 and 자경전 share the east flank, and at -34 their compound walls
/// overlapped by two blocks — the two yards ran into one another with no gap
/// between. Moved south far enough to separate them and to leave a gap in the
/// east wall wide enough for 건춘문.
const DONGGUNG_Z: i32 = -s(26);

/// Half-extents of the throne hall's two 월대 terraces.
const WOLDAE_OUTER: (i32, i32) = (s(15), s(12));
const WOLDAE_INNER: (i32, i32) = (s(12), s(9));

/// Build 경복궁 at the centre of the map: a walled precinct entered from the
/// south through 광화문, with 근정전 raised on its 월대 terraces at the north end
/// and a stone-paved court between them.
fn place_palace(world: &mut World, gy: i32) {
    let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
    lay_courtyard(world, cx, cz, gy);
    build_wall(world, cx, cz, gy);
    place_gate(world, cx, cz + PALACE_SOUTH, gy);
    // The three lesser gates, so the precinct is not a walled box with one door.
    place_wall_gate(world, cx - PALACE_X, cz + YEONGCHU_Z, gy, false); // 영추문
    place_wall_gate(world, cx + PALACE_X, cz + GEONCHUN_Z, gy, false); // 건춘문
    place_wall_gate(world, cx, cz - PALACE_NORTH, gy, true); // 신무문

    // The inner court, on the central axis: 근정문 in its south side, the ranked
    // stones down the middle, 근정전 at the head of it, and 회랑 all the way
    // round. Built in that order so the cloister's corners overwrite the ends of
    // the gate rather than the other way about.
    let court_z = cz + COURT_OFFSET_Z;
    // The outer approach: 흥례문 partway up it, then 금천 and 영제교 in the
    // stretch between that gate and 근정문 — gate, water, gate, as you meet them.
    place_timber_gate(world, cx, cz + HEUNGNYE_Z, gy, s(8), s(2));
    lay_geumcheon(world, cx, cz + GEUMCHEON_Z, gy);

    place_rank_stones(world, cx, court_z, gy);
    place_throne_hall(world, cx, court_z - s(2), gy);
    lay_cloister(world, cx, court_z, gy);
    place_timber_gate(world, cx, court_z + COURT_Z, gy, s(7), s(2));

    // 침전 — the halls the court actually lived in, running north behind the
    // throne hall, each in its own walled yard.
    place_inner_quarters(world, cx, cz, gy);

    // 경회루 — the banquet pavilion standing on its pond, west of the axis, in
    // the strip between the court's cloister and the precinct wall.
    place_gyeonghoeru(world, cx - s(30), cz - s(8), gy);
    // 자경전 — the dowager queen's hall, in the matching strip to the east.
    place_jagyeongjeon(world, cx + JAGYEONG_X, cz + JAGYEONG_Z, gy);
    // 수정전 and 동궁 fill the flanks either side of the inner yards, which were
    // bare ground between the cloister and the precinct wall.
    compound_wall(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, s(7), s(8), false);
    place_residence(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, s(5), s(4), true);
    compound_wall(world, cx + DONGGUNG_X, cz + DONGGUNG_Z, gy, s(7), s(8), false);
    place_residence(world, cx + DONGGUNG_X, cz + DONGGUNG_Z, gy, s(5), s(4), true);
    // 향원정 — the hexagonal pavilion in the rear garden, at the far north.
    place_hyangwonjeong(world, cx, cz + HYANGWON_Z, gy);

    // Last, so every gateway it has to meet is already standing. Paths are laid
    // at ground level only, so this cannot disturb any of them.
    lay_paths(world, cx, cz, gy);
}

/// Height of every 담장 in the palace, from footing to coping. Shared by the
/// precinct wall, the compound walls and the cross walls, which are the same
/// wall in three places and used to state their four courses separately — so
/// rescaling one of them silently left the others behind.
const WALL_H: i32 = s(4);

/// 주칸 — the spacing of the columns along a hall's front. Scaling this with
/// everything else keeps the *number* of bays roughly constant and makes each
/// one wider, which is what a bigger hall should look like; leaving it at 3
/// would have crowded the same slim columns closer and closer together.
const BAY: i32 = s(3);

/// Clear height of a doorway. A person is under two blocks tall, so this is
/// generous at any scale — it grows anyway so the openings stay in proportion
/// to the walls they are cut through.
const DOOR_H: i32 = s(3);

/// 처마 — how far a hall's eaves project past its walls. Korean eaves overhang
/// hard, and this is the number that carries it.
const EAVES: i32 = s(2);

/// How many courses of 공포 stand between the beam and the eaves.
///
/// Tied to `EAVES` rather than chosen: the brackets exist to walk the eave line
/// outward from the wall, so they step out one block per course and stop one
/// short, leaving the eave itself to cantilever the last block the way a real
/// 서까래 does.
const BRACKET_TIERS: i32 = EAVES - 1;

/// Spacing of the bracket sets along a run. 경복궁's halls are 다포계 — brackets
/// both on the columns and in the spaces between them — so this is tighter than
/// `BAY`. A lesser building would be 주심포, with a set only over each column.
const BRACKET_SPACING: i32 = 2;

/// 공포 — the stepped bracket sets that carry the eaves out past the wall.
///
/// Without them the roof simply appeared, three blocks wider than the building,
/// with nothing between the painted beam and the overhang: the eave hung in the
/// air. The brackets are what actually holds a Korean roof out that far, and
/// they are the densest, most recognisable band on the whole elevation.
///
/// Each course steps out one block and is drawn as a ring, painted where a
/// bracket set sits and left as plain timber between, so the band reads as a
/// rhythm rather than as a solid collar. Corners are always painted: 귀공포, the
/// corner set, is the largest one on the building and carries the two eave lines
/// that meet there.
///
/// Returns the level the roof should start at.
fn lay_brackets(world: &mut World, cx: i32, cz: i32, bx: i32, bz: i32, base_y: i32) -> i32 {
    for tier in 0..BRACKET_TIERS {
        let (ex, ez) = (bx + tier + 1, bz + tier + 1);
        let y = base_y + tier;
        for dz in -ez..=ez {
            for dx in -ex..=ex {
                let on_x_run = dz.abs() == ez;
                let on_z_run = dx.abs() == ex;
                if !on_x_run && !on_z_run {
                    continue; // the middle is the hall's ceiling, not brackets
                }
                // Measure the rhythm along whichever run this cell belongs to.
                let along = if on_x_run { dx } else { dz };
                let bracket =
                    (on_x_run && on_z_run) || along.rem_euclid(BRACKET_SPACING) == 0;
                let block = if bracket {
                    Block::Dancheong
                } else {
                    Block::Wood
                };
                world.set(cx + dx, y, cz + dz, block);
            }
        }
    }
    base_y + BRACKET_TIERS
}

/// Lay one course-by-course 담장 cell: granite footing, plaster body, tiled
/// coping. `accent` replaces the lowest body course, which is where 자경전's
/// 꽃담 carries its pattern.
fn wall_column(world: &mut World, x: i32, gy: i32, z: i32, accent: Option<Block>) {
    world.set(x, gy + 1, z, Block::Granite);
    world.set(x, gy + 2, z, accent.unwrap_or(Block::Plaster));
    for h in 3..WALL_H {
        world.set(x, gy + h, z, Block::Plaster);
    }
    world.set(x, gy + WALL_H, z, Block::RoofTile);
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
            let accent = (flowered && dz == rz).then_some(Block::Dancheong);
            wall_column(world, cx + dx, gy, cz + dz, accent);
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
    compound_wall(world, cx, cz, gy, s(7), s(11), true);
    place_residence(world, cx, cz, gy, s(5), s(4), true);

    // 십장생 굴뚝 — the tall decorated chimney standing in the yard behind.
    let chimney_z = cz - s(9);
    for h in 1..=s(6) {
        world.set(cx, gy + h, chimney_z, Block::ClayWall);
    }
    world.set(cx, gy + s(6) + 1, chimney_z, Block::Dancheong);
    world.set(cx, gy + s(6) + 2, chimney_z, Block::RoofTile);
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
    const POND_R: i32 = s(9);
    const ISLAND_R: i32 = s(4);
    const HEX_R: i32 = s(3);
    const DEPTH: i32 = s(2);

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
            for h in 2..=s(10) {
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
    for h in 1..=s(3) {
        for dz in -HEX_R..=HEX_R {
            for dx in -HEX_R..=HEX_R {
                let edge = in_hex(dx, dz, HEX_R) && !in_hex(dx, dz, HEX_R - 1);
                if edge && (dz.abs() == HEX_R || dx.abs() == HEX_R - dz.abs() / 2) {
                    world.set(cx + dx, floor + h, cz + dz, Block::RedPillar);
                }
            }
        }
    }
    let beam = floor + s(4);
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
        for dx in -s(1)..=s(1) {
            world.set(cx + dx, gy + 1, cz + dz, Block::Wood);
            for h in 2..=s(5) {
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
                if dz == rz && dx.abs() <= s(3) {
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
const INNER_X: i32 = s(20);

/// Lay out the sequence of halls north of the throne hall: 사정전 where the king
/// held council, then 강녕전 and 교태전 where he and the queen slept, each behind
/// its own cross wall.
fn place_inner_quarters(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // 사정전 keeps a ridge; it is a hall of state like the ones to the south.
    cross_wall(world, cx, cz + SAJEONG_Z + s(12), gy);
    place_residence(world, cx, cz + SAJEONG_Z, gy, s(7), s(4), true);

    // 강녕전 and 교태전 are 무량각 — built deliberately *without* a ridge beam
    // over the rooms where the king and queen slept.
    cross_wall(world, cx, cz + GANGNYEONG_Z + s(8), gy);
    place_residence(world, cx, cz + GANGNYEONG_Z, gy, s(8), s(4), false);

    cross_wall(world, cx, cz + GYOTAE_Z + s(8), gy);
    place_residence(world, cx, cz + GYOTAE_Z, gy, s(7), s(4), false);

    // 아미산 — the terraced garden behind the queen's hall. Each step is both
    // further north and one course higher, so the ground climbs away from the
    // hall towards the back wall rather than towards it.
    for step in 0..s(4) {
        let dz = GYOTAE_Z - s(4) - step;
        for dx in -s(12)..=s(12) {
            for h in 0..=step {
                world.set(cx + dx, gy + 1 + h, cz + dz, Block::Granite);
            }
        }
    }
}

/// How far off the axis the 협문 — the side gates — sit, and with them the
/// route that skirts each hall.
///
/// Every 기단 runs right up to the cross wall behind it, with nothing between:
/// the halls have 15 to 17 blocks of clear yard down either flank and *zero*
/// north of them. So a way round a hall can leave the axis and pass its side,
/// but it has no way back to the axis afterwards unless the wall it meets opens
/// somewhere other than the middle. Hence a gate either side, clear of the
/// widest platform (강녕전's, at 15) and well inside the yard's half-width of 30.
const BYPASS_X: i32 = s(13);

/// A cross wall dividing one yard from the next, with a gateway on the axis and
/// a 협문 either side of it.
fn cross_wall(world: &mut World, cx: i32, cz: i32, gy: i32) {
    for dx in -INNER_X..=INNER_X {
        let side_gate = (dx.abs() - BYPASS_X).abs() <= 1;
        if dx.abs() <= 1 || side_gate {
            // The gateways — leave them open, but carry the coping across so
            // the wall still reads as one continuous run.
            world.set(cx + dx, gy + WALL_H, cz, Block::RoofTile);
            continue;
        }
        wall_column(world, cx + dx, gy, cz, None);
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
    const BODY_H: i32 = s(4);
    /// How far the 기단 projects past the walls.
    const APRON: i32 = s(2);

    // 기단 — the platform, projecting a little past the walls all round.
    for dz in -(bz + APRON)..=(bz + APRON) {
        for dx in -(bx + APRON)..=(bx + APRON) {
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            for h in 2..=(BODY_H + s(8)) {
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
                    || (dx.rem_euclid(BAY) == 0 && dz.abs() == bz)
                    || (dz.rem_euclid(BAY) == 0 && dx.abs() == bx);
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
    for h in 0..DOOR_H {
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
    let eave = lay_brackets(world, cx, cz, bx, bz, beam + 1);
    lay_roof(world, cx, cz, bx, bz, eave, EAVES, PALACE_STEP, ridged);

    // 드므 — the bronze vats that stood at a hall's corners, kept full of water
    // as a charm against fire.
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (x, z) = (cx + sx * (bx + APRON), cz + sz * (bz + APRON));
            world.set(x, gy + 2, z, Block::Granite);
            world.set(x, gy + 3, z, Block::Water);
        }
    }
}

// --- 회랑 (the cloister) ----------------------------------------------------

/// Height of the cloister's colonnade, from its raised floor to the beam.
const CLOISTER_H: i32 = s(3);

/// Run 회랑 around all four sides of the court.
fn lay_cloister(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // North and south runs, along X; then east and west, along Z. `inward`
    // points at the court, so each run knows which of its two faces is the open
    // colonnade and which is the solid outer wall.
    for side in [-1, 1] {
        // 사정문 — the north run is the only one that opens. The axis is meant
        // to run *through* the court and out the far side to 사정전; sealed, it
        // forced you back down the length of the yard and around the outside of
        // the whole cloister, which is the one thing the layout is arranged to
        // stop you doing. The south run stays closed here because 근정문 is
        // built over it afterwards and cuts its own opening.
        let gated = side == -1;
        cloister_run(world, cx, cz + side * COURT_Z, COURT_X, true, -side, gy, gated);
        cloister_run(world, cx + side * COURT_X, cz, COURT_Z, false, -side, gy, false);
    }
}

/// One straight run of cloister: a raised walkway, solid on the outside, open
/// colonnade on the court side, under a tiled roof.
#[allow(clippy::too_many_arguments)]
fn cloister_run(
    world: &mut World,
    cx: i32,
    cz: i32,
    half_len: i32,
    along_x: bool,
    inward: i32,
    gy: i32,
    gated: bool,
) {
    let at = |t: i32, w: i32| {
        if along_x {
            (cx + t, cz + w)
        } else {
            (cx + w, cz + t)
        }
    };

    // Half-width of the walkway. The runs are laid on the court boundary, so
    // this is also how far the cloister reaches either side of it.
    const HALF_W: i32 = s(1) + 1;

    for t in -half_len..=half_len {
        // A raised granite walkway centred on the court boundary.
        for w in -HALF_W..=HALF_W {
            let (x, z) = at(t, w);
            world.set(x, gy + 1, z, Block::Granite);
            for h in 2..=(CLOISTER_H + s(5)) {
                world.set(x, gy + h, z, Block::Air);
            }
        }

        // Posts every bay. The outer face is walled between them; the court
        // face is left open, which is what makes it a colonnade and not a
        // corridor you cannot see out of.
        let post = t.rem_euclid(BAY) == 0;
        // The gateway through the outer face. Its jambs stay — they are what
        // makes it read as a doorway rather than as a hole in the wall.
        let doorway = gated && t.abs() < s(2);
        for h in 2..=(CLOISTER_H + 1) {
            let (ox, oz) = at(t, -inward * HALF_W);
            if doorway {
                world.set(ox, gy + h, oz, Block::Air);
            } else {
                world.set(
                    ox,
                    gy + h,
                    oz,
                    if post { Block::RedPillar } else { Block::Plaster },
                );
            }
            if post && !doorway {
                let (ix, iz) = at(t, inward * HALF_W);
                world.set(ix, gy + h, iz, Block::RedPillar);
            }
        }

        // Painted beam over both faces.
        let beam = gy + CLOISTER_H + 2;
        for w in [-HALF_W, HALF_W] {
            let (x, z) = at(t, w);
            world.set(x, beam, z, Block::Dancheong);
        }
    }

    // One roof over the whole run. Half-extents are in world space, so they swap
    // with the run's direction. The pitch stays at 1 — a corridor this narrow
    // would be capped in a single course at the palace step of 2.
    let (bx, bz) = if along_x {
        (half_len, HALF_W)
    } else {
        (HALF_W, half_len)
    };
    lay_roof(world, cx, cz, bx, bz, gy + CLOISTER_H + 3, 1, 1, true);
}

// --- 궁문 (the secondary gates in the precinct wall) ------------------------

/// Where 영추문, 건춘문 and 신무문 stand, as offsets from the precinct centre.
/// The two side gates sit in the gaps between the compounds along each flank —
/// 수정전 and 경회루's pond on the west, 자경전 and 동궁 on the east.
const YEONGCHU_Z: i32 = -s(24);
const GEONCHUN_Z: i32 = -s(38);

/// 영추문 / 건춘문 / 신무문 — a gate through the precinct wall.
///
/// Until now 광화문 was the only way in or out of a walled precinct 114 by 200
/// blocks: three sides were unbroken from end to end. These are the lesser
/// gates, one passage rather than 광화문's three and a much lighter base, which
/// is what they are — the everyday doors into the palace, not the ceremonial
/// one.
///
/// `along_x` says which way the wall runs here, since the gate has to lie in it
/// either way round: the north gate spans X, the two flank gates span Z.
fn place_wall_gate(world: &mut World, cx: i32, cz: i32, gy: i32, along_x: bool) {
    /// Half-width along the wall, and half-depth across it. Nine blocks
    /// across is as wide as the gaps between the flank compounds allow, and
    /// about right for a gate that is not the ceremonial one.
    const HALF: i32 = s(3);
    const THICK: i32 = s(2);
    const BASE_H: i32 = s(4);
    const PASS: i32 = s(1);

    let at = |t: i32, w: i32| {
        if along_x {
            (cx + t, cz + w)
        } else {
            (cx + w, cz + t)
        }
    };

    // A granite base filling the wall's thickness, then the passage cut back
    // out of it. Building solid first means the gate replaces whatever run of
    // 담장 it lands on rather than having to be fitted around it.
    for t in -HALF..=HALF {
        for w in -THICK..=THICK {
            let (x, z) = at(t, w);
            for h in 1..=BASE_H {
                world.set(x, gy + h, z, Block::Granite);
            }
        }
    }
    for t in -PASS..=PASS {
        for w in -THICK..=THICK {
            let (x, z) = at(t, w);
            for h in 1..=(BASE_H - 1) {
                // Round the head of the opening so it reads as an arch.
                if h == BASE_H - 1 && t.abs() == PASS {
                    continue;
                }
                world.set(x, gy + h, z, Block::Air);
            }
        }
    }

    // The painted storey over it, and its roof.
    let floor = gy + BASE_H + 1;
    const STOREY_H: i32 = s(2);
    for h in 0..STOREY_H {
        for t in -HALF..=HALF {
            for w in -THICK..=THICK {
                if t.abs() != HALF && w.abs() != THICK {
                    continue;
                }
                let (x, z) = at(t, w);
                let post = t.rem_euclid(BAY) == 0 || (t.abs() == HALF && w.abs() == THICK);
                world.set(
                    x,
                    floor + h,
                    z,
                    if post { Block::RedPillar } else { Block::Paper },
                );
            }
        }
    }
    let beam = floor + STOREY_H;
    for t in -HALF..=HALF {
        for w in -THICK..=THICK {
            if t.abs() == HALF || w.abs() == THICK {
                let (x, z) = at(t, w);
                world.set(x, beam, z, Block::Dancheong);
            }
        }
    }
    let (bx, bz) = if along_x {
        (HALF, THICK)
    } else {
        (THICK, HALF)
    };
    let eave = lay_brackets(world, cx, cz, bx, bz, beam + 1);
    lay_roof(world, cx, cz, bx, bz, eave, EAVES, PALACE_STEP, true);
}

// --- 흥례문 권역 (the outer approach) ---------------------------------------

/// 흥례문, midway up the approach, and 금천 with 영제교 over it just inside.
const HEUNGNYE_Z: i32 = s(20);
const GEUMCHEON_Z: i32 = s(12);

/// 금천 — the stream every Joseon palace puts across its entrance, and 영제교,
/// the stone bridge that carries the axis over it.
///
/// It is not decoration. You were meant to cross running water on the way in,
/// and the approach from 광화문 was 42 blocks of unbroken paving before this:
/// the eye had nothing to stop on between the outer gate and 근정문, which is
/// exactly what the real sequence of gate, water, bridge, gate exists to
/// prevent.
///
/// The channel is cut *below* the pavement rather than flooded to it, so the
/// water reads as a ditch you cross rather than as a puddle on the courtyard.
fn lay_geumcheon(world: &mut World, cx: i32, cz: i32, gy: i32) {
    /// Half-width of the water, and of the dressed stone kerb either side.
    const CHANNEL: i32 = s(2);
    const DEPTH: i32 = s(2);
    /// Half-width of the bridge deck — wider than the 삼도 it carries, so the
    /// processional way crosses without pinching.
    const DECK: i32 = s(4);

    for dz in -(CHANNEL + 1)..=(CHANNEL + 1) {
        for dx in -(PALACE_X - 1)..=(PALACE_X - 1) {
            let (x, z) = (cx + dx, cz + dz);
            if dz.abs() > CHANNEL {
                world.set(x, gy, z, Block::Granite); // the kerb
                continue;
            }
            // Cut the channel: open at pavement level, water below it. The
            // clearing has to reach *above* the pavement as well, because the
            // 삼도's centre lane stands a block proud and the courtyard is
            // already laid by the time this runs — leave it and the
            // processional way carries on straight over the water as a
            // one-block ribbon of granite with nothing holding it up.
            for h in 0..=1 {
                world.set(x, gy + h, z, Block::Air);
            }
            for d in 1..=DEPTH {
                world.set(x, gy - d, z, Block::Water);
            }
            world.set(x, gy - DEPTH - 1, z, Block::Granite); // dressed bed
        }
    }

    // 영제교 — the deck, level with the pavement either side so the crossing is
    // continuous, with the raised centre lane of the 삼도 carried over it.
    for dz in -(CHANNEL + 1)..=(CHANNEL + 1) {
        for dx in -DECK..=DECK {
            world.set(cx + dx, gy, cz + dz, Block::Granite);
            if dx == 0 {
                world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            }
        }
    }
}

/// A timber gate on the axis: a granite sill carrying a red-pillared storey
/// with three doorways in each face, under its own bracketed roof.
///
/// Both 근정문 and 흥례문 are this building at different sizes, so it takes its
/// half-extents rather than fixing them. 광화문 is *not* — it is a fortified
/// stone base with passages cut through it, which is why `place_gate` builds
/// something else entirely.
fn place_timber_gate(world: &mut World, cx: i32, cz: i32, gy: i32, gx: i32, gz: i32) {
    const BODY_H: i32 = s(4);

    for dz in -gz..=gz {
        for dx in -gx..=gx {
            world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            for h in 2..=(BODY_H + s(6)) {
                world.set(cx + dx, gy + h, cz + dz, Block::Air);
            }
        }
    }

    let floor = gy + 2;
    for h in 0..BODY_H {
        let y = floor + h;
        for dz in -gz..=gz {
            for dx in -gx..=gx {
                if dx.abs() != gx && dz.abs() != gz {
                    continue; // the passage through the middle stays open
                }
                // Three doorways in each face: the king's in the centre, an
                // officials' door either side.
                let doorway = dz.abs() == gz
                    && h < DOOR_H
                    && (dx.abs() <= s(1) || (s(4)..=s(5)).contains(&dx.abs()));
                if doorway {
                    world.set(cx + dx, y, cz + dz, Block::Air);
                    continue;
                }
                let post = dx.rem_euclid(BAY) == 0 || (dx.abs() == gx && dz.abs() == gz);
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
    for dz in -gz..=gz {
        for dx in -gx..=gx {
            if dx.abs() == gx || dz.abs() == gz {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    let eave = lay_brackets(world, cx, cz, gx, gz, beam + 1);
    lay_roof(world, cx, cz, gx, gz, eave, EAVES, PALACE_STEP, true);
}

/// 품계석 — the ranked stones officials lined up beside, in two rows down the
/// court flanking the 삼도.
fn place_rank_stones(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let mut z = cz + COURT_Z - s(5);
    while z > cz - COURT_Z + s(8) {
        for dx in [-s(5), s(5)] {
            world.set(cx + dx, gy + 1, z, Block::Granite);
        }
        z -= s(3);
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
    const POND_X: i32 = s(7);
    const POND_Z: i32 = s(12);
    const DEPTH: i32 = s(2);
    /// Half-extents of the pavilion's stone understructure.
    const BASE_X: i32 = s(4);
    const BASE_Z: i32 = s(4);

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
    let floor = gy + DEPTH + 1;
    for dz in -BASE_Z..=BASE_Z {
        for dx in -BASE_X..=BASE_X {
            world.set(cx + dx, floor, cz + dz, Block::Wood);
            for h in 1..=s(8) {
                world.set(cx + dx, floor + h, cz + dz, Block::Air);
            }
        }
    }

    // Open colonnade: columns only, no infill.
    for h in 1..=s(3) {
        for dz in -BASE_Z..=BASE_Z {
            for dx in -BASE_X..=BASE_X {
                let edge = dx.abs() == BASE_X || dz.abs() == BASE_Z;
                if edge && dx.rem_euclid(2) == 0 && dz.rem_euclid(2) == 0 {
                    world.set(cx + dx, floor + h, cz + dz, Block::RedPillar);
                }
            }
        }
    }
    let beam = floor + s(4);
    for dz in -BASE_Z..=BASE_Z {
        for dx in -BASE_X..=BASE_X {
            if dx.abs() == BASE_X || dz.abs() == BASE_Z {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    let eave = lay_brackets(world, cx, cz, BASE_X, BASE_Z, beam + 1);
    lay_roof(world, cx, cz, BASE_X, BASE_Z, eave, EAVES, PALACE_STEP, true);

    // A causeway east to the bank, at deck height. It starts *beyond* the
    // pavilion's own edge: running it from `BASE_X` cleared the colonnade and
    // deck it was supposed to join, leaving the pavilion open on that side.
    for dx in (BASE_X + 1)..=POND_X {
        world.set(cx + dx, floor, cz, Block::Granite);
        for h in 1..=s(4) {
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
    let paved_north = COURT_OFFSET_Z - COURT_Z - s(2);
    for dz in paved_north..=PALACE_SOUTH {
        for dx in -PALACE_X..=PALACE_X {
            world.set(cx + dx, gy, cz + dz, Block::Granite);
        }
    }
    for dz in (COURT_OFFSET_Z - s(2))..=PALACE_SOUTH {
        for dx in -s(3)..=s(3) {
            // The centre lane sits a block proud of the two flanking it.
            let block = if dx == 0 { Block::Granite } else { Block::Stone };
            world.set(cx + dx, gy, cz + dz, block);
            if dx == 0 {
                world.set(cx + dx, gy + 1, cz + dz, Block::Granite);
            }
        }
    }
}

// --- 어도 (the paths between the halls) -------------------------------------

/// Half-width of the paths through the residential yards. The ceremonial 삼도
/// down the south court is wider; these are working routes between one hall and
/// the next.
const PATH_W: i32 = s(1);

/// Pave a straight run at ground level, from `(x0,z0)` to `(x1,z1)`. One of the
/// two axes must be constant — palace paths turn square corners, they do not
/// wander diagonally.
///
/// Everything is written at `gy` and nothing above it, which is what makes this
/// safe to run after the buildings: walls foot at `gy + 1` and platforms at
/// `gy + 1`, so a path can only ever replace the ground *under* a structure,
/// never cut through one. Where it runs beneath a wall it is simply invisible;
/// where it passes a gateway it shows through, which is the whole point.
fn pave(world: &mut World, gy: i32, x0: i32, z0: i32, x1: i32, z1: i32, block: Block) {
    debug_assert!(x0 == x1 || z0 == z1, "paths run square, not diagonally");
    for x in x0.min(x1)..=x0.max(x1) {
        for z in z0.min(z1)..=z0.max(z1) {
            // Widen across whichever axis the run is *not* travelling along.
            let (wx, wz) = if x0 == x1 { (PATH_W, 0) } else { (0, PATH_W) };
            for ox in -wx..=wx {
                for oz in -wz..=wz {
                    world.set(x + ox, gy, z + oz, block);
                }
            }
        }
    }
}

/// Link the halls together.
///
/// The palace was a set of buildings standing on an open field: every gateway
/// opened onto undifferentiated grass, so nothing told you where to go next and
/// the axis the whole plan is built around was invisible on the ground. These
/// are the routes the real palace uses, which is also how visitors are walked
/// round it today.
///
/// Two grades, because the palace has two. The spine carries 어도 in dressed
/// granite, continuing the 삼도 that already runs up from 광화문. The spurs off
/// it to the side compounds are 흙길, beaten earth — they were service routes,
/// not processional ones, and paving them the same would flatten the hierarchy
/// the layout depends on.
fn lay_paths(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let court_north = cz + COURT_OFFSET_Z - COURT_Z;

    // The spine: out through 사정문 and north past each hall in turn, threading
    // the gateway in every cross wall on the way. It stops in front of 교태전,
    // the last hall on the axis.
    let spine_end = cz + GYOTAE_Z + s(4) + 2;
    pave(world, gy, cx, court_north, cx, spine_end, Block::Granite);

    // The flanking routes. The spine runs at every hall straight into its 기단
    // and stops, because the halls sit on the axis: to carry on you climb the
    // platform, walk round the building on its apron and drop off the far side.
    // That is a real way through — the apron is dressed granite and is meant to
    // be walked — but it is the ceremonial way, up onto the terrace of the hall
    // the king is sitting in, and it should not be the only one.
    //
    // These run the length of the residential quarter down both flanks, past
    // each hall and through the 협문 in every cross wall, and tie back into the
    // spine at both ends. One continuous pair rather than three separate
    // detours, which is also what the 행각 down the sides of these yards is.
    // Both ends tie back in *front* of a platform, not onto one. The spine runs
    // right up to 교태전's 기단 and is meant to; the flanks would be pointless if
    // their last block put you on the terrace they exist to avoid. There is no
    // rejoining behind 교태전 either — 아미산 is terraced across the axis with
    // its lowest step against the platform's back edge.
    let flank_south = cz + COURT_OFFSET_Z - COURT_Z - s(3);
    let flank_north = cz + GYOTAE_Z + s(4) + s(2) + 1;
    for side in [-1, 1] {
        let fx = cx + side * BYPASS_X;
        pave(world, gy, cx, flank_south, fx, flank_south, Block::Road);
        pave(world, gy, fx, flank_south, fx, flank_north, Block::Road);
        pave(world, gy, fx, flank_north, cx, flank_north, Block::Road);
    }

    // On to the rear garden. 교태전 stands on the axis and 아미산 is terraced
    // across it directly behind, so the way through turns off in front of the
    // hall, runs up its east flank clear of both, and comes back to the axis at
    // the pond — which is how you walk it in the real palace.
    let round_x = cx + s(16);
    let pond_z = cz + HYANGWON_Z + s(9);
    pave(world, gy, cx, spine_end, round_x, spine_end, Block::Road);
    pave(world, gy, round_x, spine_end, round_x, pond_z, Block::Road);
    pave(world, gy, round_x, pond_z, cx, pond_z, Block::Road);

    // Spurs to the compounds either side. Every compound gateway is in its
    // *south* face, so each spur runs out along a clear latitude below the
    // compound and then turns north into the opening. Running straight out at
    // the gateway's own latitude looked shorter and was wrong: the path arrived
    // broadside against the compound's south wall and spent its last ten blocks
    // buried under it, so on the ground it simply stopped at a wall.
    for (gate_x, gate_z) in [
        (cx + JAGYEONG_X, cz + JAGYEONG_Z + s(11)), // 자경전
        (cx + DONGGUNG_X, cz + DONGGUNG_Z + s(8)),  // 동궁
        (cx + SUJEONG_X, cz + SUJEONG_Z + s(8)),    // 수정전
    ] {
        let approach = gate_z + s(4);
        pave(world, gy, cx, approach, gate_x, approach, Block::Road);
        pave(world, gy, gate_x, approach, gate_x, gate_z, Block::Road);
    }

    // 경회루. Its pond fills the strip between the court's west cloister and
    // the precinct wall, and that cloister is unbroken down its whole length,
    // so there is no way west out of the court at this latitude at all — a spur
    // straight off the spine just ran under the cloister and died against it
    // from both sides. The approach comes round from the south courtyard and up
    // the pond's dressed east bank instead, which is how you walk it today.
    let bank_x = cx - s(30) + s(7);
    let below_pond = cz - s(8) + s(12) + 2;
    pave(world, gy, cx, below_pond, bank_x, below_pond, Block::Road);
    pave(world, gy, bank_x, below_pond, bank_x, cz - s(8), Block::Road);
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
            if dz == PALACE_SOUTH && dx.abs() <= s(7) {
                continue;
            }
            wall_column(world, cx + dx, gy, cz + dz, None);
        }
    }
}

/// 광화문 — the main gate: a granite base pierced by three arched passages,
/// carrying a painted timber storey and a tiled roof.
fn place_gate(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const GX: i32 = s(8); // half-width
    const GZ: i32 = s(3); // half-depth
    const BASE_H: i32 = s(5);

    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            for h in 1..=BASE_H {
                world.set(cx + dx, gy + h, cz + dz, Block::Granite);
            }
        }
    }

    // Three passages through the base. The middle one — the king's — is taller.
    for (centre, height) in [(-s(5), s(3)), (0, s(4)), (s(5), s(3))] {
        for dz in -GZ..=GZ {
            for dx in -s(1)..=s(1) {
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
    const STOREY_H: i32 = s(3);
    for h in 0..STOREY_H {
        let y = floor + h;
        for dz in -GZ..=GZ {
            for dx in -GX..=GX {
                if dx.abs() != GX && dz.abs() != GZ {
                    continue;
                }
                let post = dx.rem_euclid(s(4)) == 0 || dz.abs() == GZ && dx.abs() == GX;
                world.set(
                    cx + dx,
                    y,
                    cz + dz,
                    if post { Block::RedPillar } else { Block::Paper },
                );
            }
        }
    }
    let beam = floor + STOREY_H;
    for dz in -GZ..=GZ {
        for dx in -GX..=GX {
            if dx.abs() == GX || dz.abs() == GZ {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    let eave = lay_brackets(world, cx, cz, GX, GZ, beam + 1);
    lay_roof(world, cx, cz, GX, GZ, eave, EAVES, PALACE_STEP, true);
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
    hall_storey(world, cx, cz, s(9), s(6), floor, s(4));
    lay_hall_floor(world, cx, cz, s(9), s(6), floor - 1);
    place_throne(world, cx, cz - s(2), floor);
    let lower_beam = floor + s(4);
    let lower_eave = lay_brackets(world, cx, cz, s(9), s(6), lower_beam + 1);
    lay_roof(world, cx, cz, s(9), s(6), lower_eave, EAVES, PALACE_STEP, true);

    // Upper storey rising through the lower roof — the 중층 that makes 근정전
    // read as a throne hall rather than a large shed. It starts above the lower
    // roof's first two courses so it emerges from them instead of being buried.
    let upper_floor = lower_beam + s(4);
    hall_storey(world, cx, cz, s(6), s(4), upper_floor, s(3));
    let upper_beam = upper_floor + s(3);
    let upper_eave = lay_brackets(world, cx, cz, s(6), s(4), upper_beam + 1);
    lay_roof(world, cx, cz, s(6), s(4), upper_eave, EAVES, PALACE_STEP, true);
}

/// 어좌 — the throne, on its dais at the north end of the hall, under a 닫집
/// canopy and in front of the 일월오봉도 screen.
///
/// The hall was an empty shell until now: you could walk in through 광화문, up
/// the 삼도, through 근정문 and into the building, and find nothing at all. This
/// is what the whole axis points at.
fn place_throne(world: &mut World, cx: i32, cz: i32, floor: i32) {
    // The dais, stepping up twice.
    for dz in -s(2)..=s(1) {
        for dx in -s(3)..=s(3) {
            world.set(cx + dx, floor, cz + dz, Block::Granite);
        }
    }
    for dz in -s(2)..=0 {
        for dx in -s(2)..=s(2) {
            world.set(cx + dx, floor + 1, cz + dz, Block::Granite);
        }
    }
    world.set(cx, floor + 2, cz - s(1), Block::RedPillar); // the seat

    // 일월오봉도 — the sun, moon and five peaks, which stood behind the throne
    // wherever the king sat. The painting itself is far below this resolution;
    // what carries is a band of colour filling the wall right behind the seat.
    for dx in -s(3)..=s(3) {
        for h in 2..=s(4) {
            world.set(cx + dx, floor + h, cz - s(3), Block::Dancheong);
        }
    }

    // 닫집 — the canopy, on four posts over the seat.
    for sx in [-s(2), s(2)] {
        for sz in [-s(2), s(1)] {
            for h in 3..=s(4) {
                world.set(cx + sx, floor + h, cz + sz, Block::RedPillar);
            }
        }
    }
    for dz in -s(2)..=s(1) {
        for dx in -s(2)..=s(2) {
            world.set(cx + dx, floor + s(4) + 1, cz + dz, Block::RoofTile);
        }
    }
    for dx in -1..=1 {
        world.set(cx + dx, floor + s(4) + 2, cz, Block::RoofRidge);
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
                let bay = dx.rem_euclid(BAY) == 0 && dz.abs() == bz
                    || dz.rem_euclid(BAY) == 0 && dx.abs() == bx;
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
    for h in 0..DOOR_H.min(height) {
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
    use crate::world::{PLAY_MARGIN, WORLD_Y};

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
        // Walk the same chain of offsets the builders use, rather than the
        // absolute numbers they happened to produce at the original scale:
        // the court, then the hall set back inside it, then the throne set
        // back inside that.
        let hz = cz + COURT_OFFSET_Z - s(2);
        let tz = hz - s(2);
        let floor = GROUND + 3;
        assert_eq!(
            w.get(cx, floor + 2, tz - s(1)),
            Block::RedPillar,
            "the throne seat is missing"
        );
        // 일월오봉도 stands behind it, and the 닫집 hangs over it.
        assert_eq!(
            w.get(cx, floor + 3, tz - s(3)),
            Block::Dancheong,
            "the 일월오봉도 screen is missing"
        );
        assert_eq!(
            w.get(cx, floor + s(4) + 2, tz),
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

    /// 공포 has to actually bridge the wall and the eave. The whole reason it
    /// exists here is that the roof used to appear three blocks wider than the
    /// building with nothing beneath the overhang, and a roof that hangs in the
    /// air is exactly as "standing" as one that doesn't, so no other test here
    /// would notice it coming back.
    #[test]
    fn the_eaves_are_carried_on_brackets() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        let hz = cz + COURT_OFFSET_Z - s(2); // 근정전, on the axis
        let bx = s(9); // its lower storey's half-width
        let beam = GROUND + 3 + s(4);

        for tier in 0..BRACKET_TIERS {
            let y = beam + 1 + tier;
            let out = bx + tier + 1;
            assert!(
                out < bx + EAVES,
                "tier {tier} reaches the eave line; nothing is left to cantilever"
            );
            let b = w.get(cx + out, y, hz);
            assert!(
                matches!(b, Block::Dancheong | Block::Wood),
                "no 공포 {out} out at tier {tier}, found {b:?} — the eave has \
                 nothing under it"
            );
            // The band steps *out* as it rises: the block beyond this tier is
            // still open at this level, and gets filled by the tier above.
            assert_eq!(
                w.get(cx + out + 1, y, hz),
                Block::Air,
                "tier {tier} is a solid collar rather than a step"
            );
        }
    }

    /// You must be able to walk from 광화문 to every hall *on a path*.
    ///
    /// Reachability alone proves nothing here: the precinct is an open field, so
    /// before any of this existed you could already get anywhere by striking out
    /// across the grass and going round the back of the cloister. A flood fill
    /// over walkable ground reached all eleven landmarks and reported 144,686
    /// cells — and the palace still had no circulation to speak of. So this
    /// walks the *paved* surface only, which is the thing that actually tells a
    /// visitor where to go.
    #[test]
    fn every_hall_is_on_the_path_network() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);

        // The surface you would actually stand on, and what it is made of.
        //
        // Paving alone is not enough to call a route connected: paths are laid
        // at ground level and walls foot one block above, so a path running
        // under a wall is continuous underfoot and completely impassable. An
        // earlier version of this test checked only the paving and happily
        // passed a spur to 경회루 that tunnelled beneath the west cloister.
        //
        // Both levels have to be considered, because the 삼도's centre lane is
        // deliberately raised a block — walking the axis means walking on top
        // of it, not beside it.
        let surface = |x: i32, z: i32| {
            [GROUND + 1, GROUND].into_iter().find_map(|y| {
                let b = w.get(x, y, z);
                (b.blocks_movement()
                    && !w.get(x, y + 1, z).blocks_movement()
                    && !w.get(x, y + 2, z).blocks_movement())
                .then_some((y, b))
            })
        };
        // 취향교 counts: a timber bridge is as much a way to 향원정 as a paved
        // one, and it is the only way onto the island.
        let open = |x: i32, z: i32| {
            surface(x, z).is_some_and(|(_, b)| {
                matches!(
                    b,
                    Block::Granite | Block::Road | Block::Stone | Block::Wood
                )
            })
        };

        // Flood the paving, starting under 광화문.
        let start = (cx, cz + PALACE_SOUTH);
        assert!(open(start.0, start.1), "광화문 itself is not walkable paving");
        let mut seen = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        seen.insert(start);
        queue.push_back(start);
        while let Some((x, z)) = queue.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                if open(n.0, n.1) && seen.insert(n) {
                    queue.push_back(n);
                }
            }
        }

        for (name, x, z) in [
            // The foot of the 월대 stairs — the terrace itself stands a block
            // up, so the ground-level target is in front of it.
            ("근정전", cx, cz + COURT_OFFSET_Z - s(2) + s(12) + 2),
            ("사정전", cx, cz + SAJEONG_Z + s(4) + 2),
            ("강녕전", cx, cz + GANGNYEONG_Z + s(4) + 2),
            ("교태전", cx, cz + GYOTAE_Z + s(4) + 2),
            ("향원정", cx, cz + HYANGWON_Z + s(9)),
            ("자경전", cx + JAGYEONG_X, cz + JAGYEONG_Z + s(11)),
            ("동궁", cx + DONGGUNG_X, cz + DONGGUNG_Z + s(8)),
            ("수정전", cx + SUJEONG_X, cz + SUJEONG_Z + s(8)),
            ("경회루", cx - s(30) + s(7), cz - s(8)),
        ] {
            assert!(
                seen.contains(&(x, z)),
                "{name} is not joined to the path network at ({x},{z})"
            );
        }
    }

    /// There is a way through the residential quarter that never sets foot on a
    /// hall's 기단.
    ///
    /// The halls stand on the axis, so the spine runs into each platform and
    /// stops; you carry on by climbing it and walking round the building. That
    /// route works and always did, which is why the network test passes with or
    /// without the flanking paths — it only asks whether you can get there, and
    /// over the terraces you can. This asks the separate question the flanks
    /// exist to answer: whether you can get there *without* walking across the
    /// terrace of an occupied hall.
    #[test]
    fn the_halls_can_be_passed_without_crossing_their_terraces() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);

        // The platforms, as laid by `place_residence`: half-extents plus apron.
        let on_terrace = |x: i32, z: i32| {
            [
                (SAJEONG_Z, s(7)),
                (GANGNYEONG_Z, s(8)),
                (GYOTAE_Z, s(7)),
            ]
            .iter()
            .any(|&(hz, bx)| {
                (x - cx).abs() <= bx + s(2) && (z - (cz + hz)).abs() <= s(4) + s(2)
            })
        };
        let open = |x: i32, z: i32| {
            !on_terrace(x, z)
                && [GROUND + 1, GROUND].into_iter().any(|y| {
                    matches!(
                        w.get(x, y, z),
                        Block::Granite | Block::Road | Block::Stone
                    ) && !w.get(x, y + 1, z).blocks_movement()
                        && !w.get(x, y + 2, z).blocks_movement()
                })
        };

        // Start on the spine just inside 사정문 and try to reach the head of the
        // quarter. The goal sits one block in front of 교태전's platform: the
        // last hall on the axis is where the flanks tie back in, since 아미산
        // fills the ground immediately behind it.
        let start = (cx, cz + COURT_OFFSET_Z - COURT_Z - s(3));
        let goal = (cx, cz + GYOTAE_Z + s(4) + s(2) + 1);
        assert!(open(start.0, start.1), "the flank junction is not paved");

        let mut seen = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        seen.insert(start);
        queue.push_back(start);
        while let Some((x, z)) = queue.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, z + dz);
                if open(n.0, n.1) && seen.insert(n) {
                    queue.push_back(n);
                }
            }
        }
        assert!(
            seen.contains(&goal),
            "no way past the halls except over their terraces"
        );
    }

    /// All four gates are ways through, not decorated bulges in the wall.
    ///
    /// A gate that lands on top of a neighbouring compound comes out looking
    /// perfectly correct from outside while its passage opens into that
    /// compound's wall — 건춘문 was first placed squarely inside 자경전's yard,
    /// and nothing but walking it would have shown that.
    #[test]
    fn every_gate_opens_through_the_wall() {
        let w = generate(1);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
        for (name, gx, gz, along_x) in [
            ("광화문", cx, cz + PALACE_SOUTH, true),
            ("영추문", cx - PALACE_X, cz + YEONGCHU_Z, false),
            ("건춘문", cx + PALACE_X, cz + GEONCHUN_Z, false),
            ("신무문", cx, cz - PALACE_NORTH, true),
        ] {
            // Straight through on the centre line, from outside the wall to
            // well inside it. Tested as "is there somewhere to stand with room
            // overhead", not "is this air": the 삼도's centre lane runs raised
            // through 광화문's passage, and walking the axis means walking on
            // top of it.
            let standable = |x: i32, z: i32| {
                [GROUND + 1, GROUND].into_iter().any(|y| {
                    w.get(x, y, z).blocks_movement()
                        && !w.get(x, y + 1, z).blocks_movement()
                        && !w.get(x, y + 2, z).blocks_movement()
                })
            };
            for d in -s(4)..=s(4) {
                let (x, z) = if along_x { (gx, gz + d) } else { (gx + d, gz) };
                assert!(
                    standable(x, z),
                    "{name}: no way through {d} from the centre of the passage"
                );
            }
        }
    }

    /// Nothing may touch the ceiling or the edge of the playable area.
    ///
    /// This is the check that scaling the palace needs. `World::set` silently
    /// drops out-of-bounds writes, so a roof that outgrew the world does not
    /// fail or warn — it comes out neatly sliced off, and every other test here
    /// still passes because the building is undeniably standing.
    #[test]
    fn the_palace_fits_in_the_world() {
        let w = generate(1337);
        let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);

        let mut highest = 0;
        for z in 0..WORLD_Z {
            for x in 0..WORLD_X {
                for y in (0..WORLD_Y).rev() {
                    if w.get(x, y, z) != Block::Air {
                        highest = highest.max(y);
                        break;
                    }
                }
            }
        }
        assert!(
            highest < WORLD_Y - 1,
            "something reaches the world ceiling at y={highest}; a roof is being clipped"
        );

        // The precinct, eaves and all, has to sit inside the walkable area —
        // otherwise the far side of the palace is behind the invisible wall.
        for (name, v, limit) in [
            ("west", cx - PALACE_X, PLAY_MARGIN),
            ("east", WORLD_X - (cx + PALACE_X), PLAY_MARGIN),
            ("north", cz - PALACE_NORTH, PLAY_MARGIN),
            ("south", WORLD_Z - (cz + PALACE_SOUTH), PLAY_MARGIN),
        ] {
            assert!(
                v > limit,
                "the {name} wall is {v} from the map edge, inside the {limit}-block margin"
            );
        }
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
