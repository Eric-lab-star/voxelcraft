use super::*;
use crate::block::Block;
use crate::world::World;

/// A walled compound around a hall, with a gateway in its south face. Set
/// `flowered` to paint that face's lower course, as 자경전's 꽃담 is.
pub(super) fn compound_wall(
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

// --- 자경전 (the dowager queen's hall) ---------------------------------------

/// 자경전 in its own walled compound, with the 꽃담 — the patterned wall the
/// real one is known for — along its south side.
pub(super) fn place_jagyeongjeon(world: &mut World, cx: i32, cz: i32, gy: i32) {
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

// --- 동궁 (the crown prince's quarters) --------------------------------------

/// 동궁 — 자선당 and 비현각 in one walled yard, one behind the other.
///
/// It was a single generic hall before, which is the one thing 동궁 is not: the
/// crown prince had a residence and, separately, a place he was taught in, and
/// the pair of them is what distinguishes this compound from every other walled
/// yard on the flanks.
///
/// Both are deliberately smaller than the halls on the axis. This is the heir's
/// establishment, not the king's, and building them at 침전 scale would have
/// them reading as more important than 사정전 across the way.
pub(super) fn place_donggung(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // Sized so the roofs clear the yard wall by three blocks rather than
    // filling it. At s(4) the eaves came within a block of the 담장 on both
    // sides and the whole compound read as a lid rather than as a yard with
    // buildings standing in it.
    const HALL_X: i32 = s(3);
    const HALL_Z: i32 = s(3);

    compound_wall(world, cx, cz, gy, DONGGUNG_RX, DONGGUNG_RZ, false);
    // 자선당 in front, 비현각 behind it. Both keep their ridge — neither is a
    // sleeping hall of the king's, so neither is 무량각.
    place_residence(world, cx, cz + JASEON_Z, gy, HALL_X, HALL_Z, true);
    place_residence(world, cx, cz + BIHYEON_Z, gy, HALL_X, HALL_Z, true);
}

// --- 태원전 (the shrine, in the north-west quarter) --------------------------

/// 태원전 — the shrine hall, standing alone at the back of a deep walled yard.
///
/// This one is not lived in and must not look as though it is. Every other
/// compound in the palace centres its hall in its yard; this one pushes the
/// hall to the *back* and leaves the whole court in front of it empty, so you
/// come through the gate and cross twenty blocks of nothing to reach it. The
/// emptiness is the building — a shrine precinct is arranged to be walked
/// through slowly, and it is the only thing here that distinguishes this from
/// another pair of quarters.
///
/// It genuinely is one hall. A subsidiary 영사재 was tried in the court and
/// filled exactly the space that does the work, and the yard is 33 blocks
/// across — too narrow to stand one beside the hall instead.
pub(super) fn place_taewonjeon(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const HALL_X: i32 = s(6);
    const HALL_Z: i32 = s(4);

    compound_wall(world, cx, cz, gy, TAEWON_RX, TAEWON_RZ, false);
    place_residence(world, cx, cz + TAEWON_HALL_Z, gy, HALL_X, HALL_Z, true);
}

// --- 건청궁 (the king's private residence, behind the garden) ----------------

/// 건청궁 — 장안당 and 곤녕합 in a walled yard north-east of 향원정.
///
/// Deliberately domestic. Everything else in the precinct is a hall of state or
/// serves one, and 건청궁 was built to be the opposite: a house, off the axis
/// and behind the garden, which the king could live in away from all of it. So
/// no 월대, no double roof, and the yard is generous rather than tight.
pub(super) fn place_geoncheongung(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const HALL_X: i32 = s(5);
    const HALL_Z: i32 = s(3);

    compound_wall(world, cx, cz, gy, GEONCHEONG_RX, GEONCHEONG_RZ, false);

    // Its gateway faces *west*, onto the garden path, rather than south like
    // every other compound here. 자경전's yard comes up to within three blocks
    // of this one's south wall, so there is no room for an approach on that
    // side at all — and the way anyone actually arrives is from the pond.
    for dz in -1..=1 {
        for h in 1..=WALL_H {
            world.set(cx - GEONCHEONG_RX, gy + h, cz + dz, Block::Air);
        }
    }

    place_residence(world, cx, cz + JANGAN_Z, gy, HALL_X, HALL_Z, true);
    place_residence(world, cx, cz + GONNYEONG_Z, gy, HALL_X, HALL_Z, true);
}

/// 행각 — a long, low service range.
///
/// Everything built so far is a hall: a body on a platform under a bracketed
/// roof, painted along its beam. A range is none of those things. It is three
/// blocks deep and as long as the yard it edges, it stands low, and — the part
/// that actually does the work of telling it apart — it carries **no 단청 and
/// no 공포**. Paint and bracket sets marked a building as important, and the
/// kitchens were not. A plain timber beam and a roof straight onto it.
pub(super) fn lay_range(
    world: &mut World,
    cx: i32,
    cz: i32,
    gy: i32,
    half_len: i32,
    along_x: bool,
) {
    const DEPTH: i32 = 1;
    const BODY_H: i32 = s(3);

    let at = |t: i32, w: i32| {
        if along_x {
            (cx + t, cz + w)
        } else {
            (cx + w, cz + t)
        }
    };

    for t in -half_len..=half_len {
        for w in -DEPTH..=DEPTH {
            let (x, z) = at(t, w);
            world.set(x, gy + 1, z, Block::Granite);
            for h in 2..=(BODY_H + s(4)) {
                world.set(x, gy + h, z, Block::Air);
            }
        }
    }

    let floor = gy + 2;
    for h in 0..BODY_H {
        for t in -half_len..=half_len {
            for w in -DEPTH..=DEPTH {
                if t.abs() != half_len && w.abs() != DEPTH {
                    continue;
                }
                let (x, z) = at(t, w);
                let post = t.rem_euclid(BAY) == 0;
                // Doors all down the length: these are stores and kitchens
                // opening onto the yard, not a hall with one way in.
                let door = w == -DEPTH && h < DOOR_H && t.rem_euclid(BAY) == s(1) + 1;
                let block = if door {
                    Block::Air
                } else if post {
                    Block::Wood
                } else {
                    Block::Plaster
                };
                world.set(x, floor + h, z, block);
            }
        }
    }

    // A plain beam, and the roof straight onto it — no brackets.
    let beam = floor + BODY_H;
    for t in -half_len..=half_len {
        for w in -DEPTH..=DEPTH {
            let (x, z) = at(t, w);
            world.set(x, beam, z, Block::Wood);
        }
    }
    let (bx, bz) = if along_x {
        (half_len, DEPTH)
    } else {
        (DEPTH, half_len)
    };
    lay_roof(world, cx, cz, bx, bz, beam + 1, 1, 1, true);
}

/// 소주방 — the kitchens, in a service yard off 강녕전's.
///
/// Ranges down two sides only. The strip is thirteen blocks wide, and putting
/// one down each side left a yard three blocks across — a corridor, not a court
/// you could work in. Open towards 강녕전, which is where the food went.
pub(super) fn place_sojubang(world: &mut World, cx: i32, cz: i32, gy: i32) {
    // The east range, running the depth of the yard.
    lay_range(world, cx + s(2), cz, gy, s(5), false);
    // The north range, closing the top of it. Its eaves reach two blocks
    // further than its platform, and at the first placement they came down over
    // the flanking path — the range stood on one of the path's three blocks and
    // overhung another.
    lay_range(world, cx - s(1), cz - s(4), gy, s(3), true);
}

/// 함화당 and 집경당 — a pair of halls in one yard, west of the axis.
///
/// Side by side rather than one behind the other, which every other paired
/// compound here does. The ground is 34 blocks across and 26 deep, so the long
/// dimension is east-west for once and the halls follow it — and the map shows
/// them abreast in any case.
///
/// They are small. Two platforms across a yard this wide leaves three blocks
/// between them and three to each wall, which is the whole budget: build them
/// at 침전 scale and the pair meets in the middle with no yard at all.
pub(super) fn place_hamhwadang(world: &mut World, cx: i32, cz: i32, gy: i32) {
    const HALL_X: i32 = s(2);
    const HALL_Z: i32 = s(3);

    compound_wall(world, cx, cz, gy, HAMHWA_RX, HAMHWA_RZ, false);
    place_residence(world, cx - HAMHWA_SPREAD, cz, gy, HALL_X, HALL_Z, true);
    place_residence(world, cx + HAMHWA_SPREAD, cz, gy, HALL_X, HALL_Z, true);
}
