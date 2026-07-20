use super::*;
use crate::block::Block;
use crate::world::World;

/// How far off the axis the 협문 — the side gates — sit, and with them the
/// route that skirts each hall.
///
/// Every 기단 runs right up to the cross wall behind it, with nothing between:
/// the halls have 15 to 17 blocks of clear yard down either flank and *zero*
/// north of them. So a way round a hall can leave the axis and pass its side,
/// but it has no way back to the axis afterwards unless the wall it meets opens
/// somewhere other than the middle. Hence a gate either side, clear of the
/// widest platform (강녕전's, at 15) and well inside the yard's half-width of 30.
pub(super) const BYPASS_X: i32 = s(13);

// --- 회랑 (the cloister) ----------------------------------------------------

/// Height of the cloister's colonnade, from its raised floor to the beam.
pub(super) const CLOISTER_H: i32 = s(3);

/// Run 회랑 around all four sides of the court.
pub(super) fn lay_cloister(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
        cloister_run(
            world,
            cx,
            cz + side * COURT_Z,
            COURT_X,
            true,
            -side,
            gy,
            gated,
        );
        cloister_run(
            world,
            cx + side * COURT_X,
            cz,
            COURT_Z,
            false,
            -side,
            gy,
            false,
        );
    }
}

/// One straight run of cloister: a raised walkway, solid on the outside, open
/// colonnade on the court side, under a tiled roof.
#[allow(clippy::too_many_arguments)]
pub(super) fn cloister_run(
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
                    if post {
                        Block::RedPillar
                    } else {
                        Block::Plaster
                    },
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

/// 품계석 — the ranked stones officials lined up beside, in two rows down the
/// court flanking the 삼도.
pub(super) fn place_rank_stones(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let mut z = cz + COURT_Z - s(5);
    while z > cz - COURT_Z + s(8) {
        for dx in [-s(5), s(5)] {
            world.set(cx + dx, gy + 1, z, Block::Granite);
        }
        z -= s(3);
    }
}

/// Pave the ceremonial half of the precinct in granite and run the 삼도 — the
/// raised processional way — from 광화문 up to the throne hall.
///
/// Only the southern, ceremonial half is paved. The residential yards behind the
/// throne hall keep the bare ground, with each hall standing on its own stone
/// platform; paving the whole precinct made it read as one enormous parade
/// ground rather than a sequence of separate courts.
pub(super) fn lay_courtyard(world: &mut World, cx: i32, cz: i32, gy: i32) {
    let paved_north = COURT_OFFSET_Z - COURT_Z - s(2);
    for dz in paved_north..=PALACE_SOUTH {
        for dx in -PALACE_X..=PALACE_X {
            world.set(cx + dx, gy, cz + dz, Block::Granite);
        }
    }
    for dz in (COURT_OFFSET_Z - s(2))..=PALACE_SOUTH {
        for dx in -s(3)..=s(3) {
            // The centre lane sits a block proud of the two flanking it.
            let block = if dx == 0 {
                Block::Granite
            } else {
                Block::Stone
            };
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
pub(super) const PATH_W: i32 = s(1);

/// Pave a straight run at ground level, from `(x0,z0)` to `(x1,z1)`. One of the
/// two axes must be constant — palace paths turn square corners, they do not
/// wander diagonally.
///
/// Everything is written at `gy` and nothing above it, which is what makes this
/// safe to run after the buildings: walls foot at `gy + 1` and platforms at
/// `gy + 1`, so a path can only ever replace the ground *under* a structure,
/// never cut through one. Where it runs beneath a wall it is simply invisible;
/// where it passes a gateway it shows through, which is the whole point.
pub(super) fn pave(world: &mut World, gy: i32, x0: i32, z0: i32, x1: i32, z1: i32, block: Block) {
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
pub(super) fn lay_paths(world: &mut World, cx: i32, cz: i32, gy: i32) {
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
    // The garden run carries on north past the turn to the bridge, because
    // 건청궁 is further up still and is entered off the side of it.
    let gc_z = cz + GEONCHEONG_Z;
    pave(world, gy, cx, spine_end, round_x, spine_end, Block::Road);
    pave(world, gy, round_x, spine_end, round_x, gc_z, Block::Road);
    pave(world, gy, round_x, pond_z, cx, pond_z, Block::Road);
    pave(
        world,
        gy,
        round_x,
        gc_z,
        cx + GEONCHEONG_X - GEONCHEONG_RX,
        gc_z,
        Block::Road,
    );

    // Spurs to the compounds either side. Every compound gateway is in its
    // *south* face, so each spur runs out along a clear latitude below the
    // compound and then turns north into the opening. Running straight out at
    // the gateway's own latitude looked shorter and was wrong: the path arrived
    // broadside against the compound's south wall and spent its last ten blocks
    // buried under it, so on the ground it simply stopped at a wall.
    for (gate_x, gate_z) in [
        (cx + JAGYEONG_X, cz + JAGYEONG_Z + s(11)),       // 자경전
        (cx + DONGGUNG_X, cz + DONGGUNG_Z + DONGGUNG_RZ), // 동궁
        (cx + SUJEONG_X, cz + SUJEONG_Z + s(8)),          // 수정전
    ] {
        let approach = gate_z + s(4);
        pave(world, gy, cx, approach, gate_x, approach, Block::Road);
        pave(world, gy, gate_x, approach, gate_x, gate_z, Block::Road);
    }

    // 소주방's yard, off the east flank. Short: the kitchens sit right against
    // the flanking route by design, being the one place in the palace that
    // wanted traffic rather than seclusion.
    pave(
        world,
        gy,
        cx + BYPASS_X,
        cz + SOJU_Z,
        cx + SOJU_X - s(2),
        cz + SOJU_Z,
        Block::Road,
    );

    // 함화당's yard, off the west flank. Its gateway is in the south face like
    // the rest, so the spur runs out below it and turns up.
    let hh_x = cx + HAMHWA_X;
    let hh_gate = cz + HAMHWA_Z + HAMHWA_RZ;
    let hh_approach = hh_gate + s(4);
    pave(world, gy, cx - BYPASS_X, hh_approach, hh_x, hh_approach, Block::Road);
    pave(world, gy, hh_x, hh_approach, hh_x, hh_gate, Block::Road);

    // 태원전, off the north end of the west flank. Its gateway is in the south
    // face, so the spur runs out level with the flank's end and then turns up
    // into it.
    let tw_x = cx + TAEWON_X;
    let tw_gate = cz + TAEWON_Z + TAEWON_RZ;
    pave(
        world,
        gy,
        cx - BYPASS_X,
        flank_north,
        tw_x,
        flank_north,
        Block::Road,
    );
    pave(world, gy, tw_x, flank_north, tw_x, tw_gate, Block::Road);

    // 경회루. Its pond fills the strip between the court's west cloister and
    // the precinct wall, and that cloister is unbroken down its whole length,
    // so there is no way west out of the court at this latitude at all — a spur
    // straight off the spine just ran under the cloister and died against it
    // from both sides. The approach comes round from the south courtyard and up
    // the pond's dressed east bank instead, which is how you walk it today.
    let bank_x = cx - s(30) + s(7);
    let below_pond = cz - s(8) + s(12) + 2;
    pave(world, gy, cx, below_pond, bank_x, below_pond, Block::Road);
    pave(
        world,
        gy,
        bank_x,
        below_pond,
        bank_x,
        cz - s(8),
        Block::Road,
    );
}
