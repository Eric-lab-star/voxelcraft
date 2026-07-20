use super::*;
use crate::block::Block;
use crate::world::World;

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
pub(super) fn place_wall_gate(world: &mut World, cx: i32, cz: i32, gy: i32, along_x: bool) {
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

// --- 동십자각 (the corner watchtower) ---------------------------------------

/// 동십자각 — the watchtower on the south-east corner of the wall.
///
/// The first thing here that is neither a hall nor a gate. Everything else in
/// the precinct is a roof on a platform inside a yard; this is a solid block of
/// masonry taller than anything around it with an open pavilion on top, and it
/// straddles the corner rather than standing behind the wall, so the wall runs
/// into it from two directions and stops.
///
/// The base has to out-top the 담장 by a clear margin or the whole thing reads
/// as a lump in the wall rather than as something built to see over it: nine
/// courses against the wall's six, and two more than 광화문's.
///
/// The pavilion is drawn in from the base's edge, leaving a walkway round it —
/// which is what the tower is *for*, and without it the roof would sit on the
/// masonry like a cap.
pub(super) fn place_corner_tower(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const HALF: i32 = s(4);
    const BASE_H: i32 = s(6);
    /// How far the pavilion stands in from the edge of the base.
    const INSET: i32 = s(1) + 1;

    for dz in -HALF..=HALF {
        for dx in -HALF..=HALF {
            for h in 1..=BASE_H {
                world.set(cx + dx, gy + h, cz + dz, Block::Granite);
            }
        }
    }

    // A stair down the inward face, so the tower can actually be climbed. It
    // runs north into the precinct, dropping a course a block, and lands on the
    // courtyard paving.
    //
    // Set well inside the wall line rather than on it. Run straight north off
    // the corner it cut a descending notch through the 담장 for its whole
    // length — and since it bottoms out a block above the ground, that notch
    // was a step up onto the wall from *outside* the palace and a walkway in.
    let stair_x = cx - HALF + 2;
    for step in 0..BASE_H {
        let z = cz - HALF - 1 - step;
        for dx in -1..=1 {
            world.set(stair_x + dx, gy + BASE_H - step, z, Block::Granite);
            for h in 1..=s(2) {
                world.set(stair_x + dx, gy + BASE_H - step + h, z, Block::Air);
            }
        }
    }

    // The pavilion: an open colonnade, no infill — it is a lookout.
    let (px, floor) = (HALF - INSET, gy + BASE_H + 1);
    for h in 0..s(3) {
        for dz in -px..=px {
            for dx in -px..=px {
                if dx.abs() != px && dz.abs() != px {
                    continue;
                }
                let corner = dx.abs() == px && dz.abs() == px;
                if corner || dx.rem_euclid(BAY) == 0 || dz.rem_euclid(BAY) == 0 {
                    world.set(cx + dx, floor + h, cz + dz, Block::RedPillar);
                }
            }
        }
    }
    let beam = floor + s(3);
    for dz in -px..=px {
        for dx in -px..=px {
            if dx.abs() == px || dz.abs() == px {
                world.set(cx + dx, beam, cz + dz, Block::Dancheong);
            }
        }
    }
    let eave = lay_brackets(world, cx, cz, px, px, beam + 1);
    lay_roof(world, cx, cz, px, px, eave, EAVES, PALACE_STEP, true);
}

/// A timber gate on the axis: a granite sill carrying a red-pillared storey
/// with three doorways in each face, under its own bracketed roof.
///
/// Both 근정문 and 흥례문 are this building at different sizes, so it takes its
/// half-extents rather than fixing them. 광화문 is *not* — it is a fortified
/// stone base with passages cut through it, which is why `place_gate` builds
/// something else entirely.
pub(super) fn place_timber_gate(world: &mut World, cx: i32, cz: i32, gy: i32, gx: i32, gz: i32) {
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

/// The 담장 around the precinct: granite footing, plaster body, tiled coping —
/// left open where 광화문 stands.
pub(super) fn build_wall(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
pub(super) fn place_gate(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
