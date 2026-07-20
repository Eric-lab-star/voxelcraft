use super::*;
use crate::block::Block;
use crate::world::World;

// --- 향원정 (the pavilion in the rear garden) --------------------------------

/// Is `(dx, dz)` inside a hexagon of radius `r`? Drawing the pavilion round
/// would waste the one chance this palace has to show a shape that isn't a
/// rectangle — 향원정 is famously six-sided.
pub(super) fn in_hex(dx: i32, dz: i32, r: i32) -> bool {
    dz.abs() <= r && dx.abs() <= r - dz.abs() / 2
}

/// A stepped roof that keeps the hexagon. Running the ordinary rectangular
/// roof over 향원정 hid the six-sided plan completely — from above, the only
/// angle the shape really shows from, it came out square like everything else.
///
/// Each course draws in by one, so consecutive bands are adjacent in plan and
/// tile without gaps, and the last is the single-block finial (절병통).
pub(super) fn lay_hex_roof(world: &mut World, cx: i32, cz: i32, r: i32, base_y: i32) {
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
pub(super) fn place_hyangwonjeong(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
pub(super) fn lay_geumcheon(world: &mut World, cx: i32, cz: i32, gy: i32) {
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

// --- 경회루 (the pavilion on the pond) --------------------------------------

/// Dig a pond and stand 경회루 in the middle of it on stone pillars, with a
/// causeway back to the bank.
///
/// The pavilion has no walls at all — it is a roof on columns, open on every
/// side, which is exactly what it was for.
pub(super) fn place_gyeonghoeru(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
    lay_roof(
        world,
        cx,
        cz,
        BASE_X,
        BASE_Z,
        eave,
        EAVES,
        PALACE_STEP,
        true,
    );

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
