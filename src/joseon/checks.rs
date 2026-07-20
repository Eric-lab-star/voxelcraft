use super::*;
use crate::block::Block;
use crate::world::{PLAY_MARGIN, WORLD_X, WORLD_Y, WORLD_Z};

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
    assert_eq!(
        crown(GANGNYEONG_Z),
        Block::RoofTile,
        "강녕전 should be 무량각"
    );
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
        // 동궁 is checked as its two halls rather than as one compound:
        // counting from the middle of the yard reaches into both, so a
        // single check there passes just as happily when one of them is
        // missing and the other has quietly grown over the gap.
        ("자선당", cx + DONGGUNG_X, cz + DONGGUNG_Z + JASEON_Z),
        ("비현각", cx + DONGGUNG_X, cz + DONGGUNG_Z + BIHYEON_Z),
        ("장안당", cx + GEONCHEONG_X, cz + GEONCHEONG_Z + JANGAN_Z),
        ("곤녕합", cx + GEONCHEONG_X, cz + GEONCHEONG_Z + GONNYEONG_Z),
        ("태원전", cx + TAEWON_X, cz + TAEWON_Z + TAEWON_HALL_Z),
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
            assert_eq!(
                b,
                Block::Air,
                "gate passage blocked at dz={dz} h={h}: {b:?}"
            );
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
    assert_eq!(
        plants, 0,
        "grass or flowers were scattered on the Joseon map"
    );
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
            matches!(b, Block::Granite | Block::Road | Block::Stone | Block::Wood)
        })
    };

    // Flood the paving, starting under 광화문.
    let start = (cx, cz + PALACE_SOUTH);
    assert!(
        open(start.0, start.1),
        "광화문 itself is not walkable paving"
    );
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
        // Entered from the west, off the garden path.
        (
            "건청궁",
            cx + GEONCHEONG_X - GEONCHEONG_RX,
            cz + GEONCHEONG_Z,
        ),
        ("태원전", cx + TAEWON_X, cz + TAEWON_Z + TAEWON_RZ),
        ("자경전", cx + JAGYEONG_X, cz + JAGYEONG_Z + s(11)),
        ("동궁", cx + DONGGUNG_X, cz + DONGGUNG_Z + DONGGUNG_RZ),
        ("수정전", cx + SUJEONG_X, cz + SUJEONG_Z + s(8)),
        ("경회루", cx + GYEONGHOE_X + s(7), cz + GYEONGHOE_Z),
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
        [(SAJEONG_Z, s(7)), (GANGNYEONG_Z, s(8)), (GYOTAE_Z, s(7))]
            .iter()
            .any(|&(hz, bx)| (x - cx).abs() <= bx + s(2) && (z - (cz + hz)).abs() <= s(4) + s(2))
    };
    let open = |x: i32, z: i32| {
        !on_terrace(x, z)
            && [GROUND + 1, GROUND].into_iter().any(|y| {
                matches!(w.get(x, y, z), Block::Granite | Block::Road | Block::Stone)
                    && !w.get(x, y + 1, z).blocks_movement()
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

/// With the four gates sealed, the wall holds all the way round.
///
/// 동십자각's stair first ran north straight off the corner, along the wall
/// line. Descending a course a block, it cut a notch clean through the 담장
/// for its whole length, and because it bottoms out one block up, that notch
/// was a step onto the wall from *outside* the palace and a walkway down
/// into it. Nothing looked wrong: the tower was correct, the wall either
/// side of it was correct, and every other test passed.
#[test]
fn the_wall_holds_between_its_gates() {
    let w = generate(1);
    let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);

    // Plug each gate mouth, generously, so what is left is only the wall.
    let gates = [
        (cx, cz + PALACE_SOUTH),
        (cx - PALACE_X, cz + YEONGCHU_Z),
        (cx + PALACE_X, cz + GEONCHUN_Z),
        (cx, cz - PALACE_NORTH),
    ];
    let sealed = |x: i32, z: i32| {
        gates
            .iter()
            .any(|&(gx, gz)| (x - gx).abs() <= s(9) && (z - gz).abs() <= s(9))
    };
    let standable = |x: i32, z: i32| {
        [GROUND + 1, GROUND].into_iter().any(|y| {
            w.get(x, y, z).blocks_movement()
                && !w.get(x, y + 1, z).blocks_movement()
                && !w.get(x, y + 2, z).blocks_movement()
        })
    };

    // Start well outside the south-east corner, where the tower is.
    let start = (cx + PALACE_X + s(8), cz + PALACE_SOUTH + s(8));
    let mut seen = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    seen.insert(start);
    queue.push_back(start);
    while let Some((x, z)) = queue.pop_front() {
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (x + dx, z + dz);
            // Keep the search in a box around the precinct.
            if (n.0 - cx).abs() > PALACE_X + s(14)
                || n.1 < cz - PALACE_NORTH - s(14)
                || n.1 > cz + PALACE_SOUTH + s(14)
            {
                continue;
            }
            if !sealed(n.0, n.1) && standable(n.0, n.1) && seen.insert(n) {
                queue.push_back(n);
            }
        }
    }

    for (name, x, z) in [
        ("the outer court", cx, cz + PALACE_SOUTH - s(10)),
        ("the west flank", cx - BYPASS_X, cz + COURT_OFFSET_Z),
        ("the rear garden", cx, cz + HYANGWON_Z + s(9)),
    ] {
        assert!(
            !seen.contains(&(x, z)),
            "got into {name} at ({x},{z}) without using a gate — the wall is breached"
        );
    }
}

/// 소주방 is built as service ranges, not as another hall.
///
/// The distinction is entirely in what it *lacks*. Paint and bracket sets
/// marked a building as important and the kitchens were not, so the moment
/// a range picks up a 단청 beam it stops reading as a service building and
/// the yard becomes one more set of quarters. That is an easy thing to lose
/// by reaching for the nearest existing builder.
#[test]
fn the_kitchens_are_ranges_not_halls() {
    let w = generate(1);
    let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
    let (ox, oz) = (cx + SOJU_X, cz + SOJU_Z);

    let mut plaster = 0;
    let mut timber = 0;
    let mut painted = 0;
    // Kept to 소주방's own footprint. Wider and it reaches 자경전, whose 공포
    // project two blocks west of its body and are painted — the count then
    // measures the neighbour and fails for the wrong reason.
    for dz in -s(6)..=s(6) {
        for dx in -s(4)..=s(4) {
            for y in GROUND..GROUND + 12 {
                match w.get(ox + dx, y, oz + dz) {
                    Block::Plaster => plaster += 1,
                    Block::Wood => timber += 1,
                    Block::Dancheong => painted += 1,
                    _ => {}
                }
            }
        }
    }
    assert!(plaster > 30, "the ranges have no walls ({plaster})");
    assert!(timber > 20, "the ranges have no timber beam ({timber})");
    assert_eq!(painted, 0, "a service range picked up a 단청 beam");
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
