use crate::block::Block;
use crate::world::World;

/// Scale a *building* — a hall's half-extents, a wall's height, an eave's
/// reach.
///
/// Kept separate from `d`, which scales the distances between things, because
/// the two were badly out of step. Measured against the real 경복궁 the
/// buildings here stood at 0.7 to 0.9 of full size while the gaps between them
/// were at 0.23 to 0.30 — nearly full-size halls packed three to four times too
/// close. That is what made the palace feel cramped, and no single scale factor
/// could fix it, because the buildings were very nearly right already.
///
/// Deliberately *not* applied to: `PALACE_STEP`, which is a ratio rather than a
/// length, so the roofs keep their 1:2 pitch and simply grow taller with the
/// halls they cover; and the 월대 terrace courses, which are one block each and
/// have no fraction to grow by.
pub(super) const fn s(n: i32) -> i32 {
    n * 2
}

/// Scale a *distance* — where a building stands, how wide a yard is, how far
/// the wall runs.
///
/// Larger than `s` on purpose. Buildings that are close to their real size want
/// the ground between them close to its real size too, and at the old common
/// factor the approach from 광화문 to 흥례문 came out eight blocks where the
/// real one is sixty. This puts it at twenty-five.
pub(super) const fn d(n: i32) -> i32 {
    n * 7 / 2
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
pub(super) fn lay_roof(
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
            world.set(
                cx + sx * eave_x,
                base_y + 1,
                cz + sz * eave_z,
                Block::RoofRidge,
            );
        }
    }
    y
}

/// Palace roofs step in 2 per course. At `step` 1 a hall this wide would carry a
/// roof taller than the building; 2 gives the shallow pitch of the real thing.
pub(super) const PALACE_STEP: i32 = 2;

/// Height of every 담장 in the palace, from footing to coping. Shared by the
/// precinct wall, the compound walls and the cross walls, which are the same
/// wall in three places and used to state their four courses separately — so
/// rescaling one of them silently left the others behind.
pub(super) const WALL_H: i32 = s(4);

/// 주칸 — the spacing of the columns along a hall's front. Scaling this with
/// everything else keeps the *number* of bays roughly constant and makes each
/// one wider, which is what a bigger hall should look like; leaving it at 3
/// would have crowded the same slim columns closer and closer together.
pub(super) const BAY: i32 = s(3);

/// Clear height of a doorway. A person is under two blocks tall, so this is
/// generous at any scale — it grows anyway so the openings stay in proportion
/// to the walls they are cut through.
pub(super) const DOOR_H: i32 = s(3);

/// 처마 — how far a hall's eaves project past its walls. Korean eaves overhang
/// hard, and this is the number that carries it.
pub(super) const EAVES: i32 = s(2);

/// How many courses of 공포 stand between the beam and the eaves.
///
/// Tied to `EAVES` rather than chosen: the brackets exist to walk the eave line
/// outward from the wall, so they step out one block per course and stop one
/// short, leaving the eave itself to cantilever the last block the way a real
/// 서까래 does.
pub(super) const BRACKET_TIERS: i32 = EAVES - 1;

/// Spacing of the bracket sets along a run. 경복궁's halls are 다포계 — brackets
/// both on the columns and in the spaces between them — so this is tighter than
/// `BAY`. A lesser building would be 주심포, with a set only over each column.
pub(super) const BRACKET_SPACING: i32 = 2;

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
pub(super) fn lay_brackets(
    world: &mut World,
    cx: i32,
    cz: i32,
    bx: i32,
    bz: i32,
    base_y: i32,
) -> i32 {
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
                let bracket = (on_x_run && on_z_run) || along.rem_euclid(BRACKET_SPACING) == 0;
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
pub(super) fn wall_column(world: &mut World, x: i32, gy: i32, z: i32, accent: Option<Block>) {
    world.set(x, gy + 1, z, Block::Granite);
    world.set(x, gy + 2, z, accent.unwrap_or(Block::Plaster));
    for h in 3..WALL_H {
        world.set(x, gy + h, z, Block::Plaster);
    }
    world.set(x, gy + WALL_H, z, Block::RoofTile);
}
