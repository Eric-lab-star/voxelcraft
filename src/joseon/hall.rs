use super::*;
use crate::block::Block;
use crate::world::World;

/// Board the inside of a hall out in timber. Without this you step through the
/// doors onto bare foundation stone and the building reads as a shell rather
/// than a room.
pub(super) fn lay_hall_floor(world: &mut World, cx: i32, cz: i32, bx: i32, bz: i32, y: i32) {
    for dz in -(bz - 1)..=(bz - 1) {
        for dx in -(bx - 1)..=(bx - 1) {
            world.set(cx + dx, y, cz + dz, Block::Wood);
        }
    }
}

/// 난간 — balustrades round both 월대 terraces, open on the axis where the
/// stairs come up. Besides being what the real terraces have, the openings turn
/// a platform you could scramble onto anywhere into one you approach the way you
/// are meant to: up the middle, facing the throne.
pub(super) fn place_terrace_rails(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
pub(super) const INNER_X: i32 = d(23);

/// Lay out the sequence of halls north of the throne hall: 사정전 where the king
/// held council, then 강녕전 and 교태전 where he and the queen slept, each behind
/// its own cross wall.
pub(super) fn place_inner_quarters(world: &mut World, cx: i32, cz: i32, gy: i32) {
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

/// A cross wall dividing one yard from the next, with a gateway on the axis and
/// a 협문 either side of it.
pub(super) fn cross_wall(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
pub(super) fn place_residence(
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

/// 근정전 — the throne hall, on two granite 월대 terraces, with the double roof
/// that gives it its silhouette.
pub(super) fn place_throne_hall(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
    lay_roof(
        world,
        cx,
        cz,
        s(9),
        s(6),
        lower_eave,
        EAVES,
        PALACE_STEP,
        true,
    );

    // Upper storey rising through the lower roof — the 중층 that makes 근정전
    // read as a throne hall rather than a large shed. It starts above the lower
    // roof's first two courses so it emerges from them instead of being buried.
    let upper_floor = lower_beam + s(4);
    hall_storey(world, cx, cz, s(6), s(4), upper_floor, s(3));
    let upper_beam = upper_floor + s(3);
    let upper_eave = lay_brackets(world, cx, cz, s(6), s(4), upper_beam + 1);
    lay_roof(
        world,
        cx,
        cz,
        s(6),
        s(4),
        upper_eave,
        EAVES,
        PALACE_STEP,
        true,
    );
}

/// 어좌 — the throne, on its dais at the north end of the hall, under a 닫집
/// canopy and in front of the 일월오봉도 screen.
///
/// The hall was an empty shell until now: you could walk in through 광화문, up
/// the 삼도, through 근정문 and into the building, and find nothing at all. This
/// is what the whole axis points at.
pub(super) fn place_throne(world: &mut World, cx: i32, cz: i32, floor: i32) {
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

pub(super) fn terrace(world: &mut World, cx: i32, cz: i32, rx: i32, rz: i32, y: i32) {
    for dz in -rz..=rz {
        for dx in -rx..=rx {
            world.set(cx + dx, y, cz + dz, Block::Granite);
        }
    }
}

/// One storey of a palace hall: red columns on a regular bay spacing, wall
/// infill between them, and a painted beam capping it.
pub(super) fn hall_storey(
    world: &mut World,
    cx: i32,
    cz: i32,
    bx: i32,
    bz: i32,
    floor: i32,
    height: i32,
) {
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
