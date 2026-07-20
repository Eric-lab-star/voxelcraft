//! 푯말 — the name boards standing in front of the buildings.
//!
//! Each is a post with a board on top. Looking at one names the building it
//! stands for; nothing is drawn until you do, so the palace is not littered
//! with floating labels.
//!
//! The names live in a table keyed by the board's own position rather than
//! being attached to the buildings, which keeps them working across a save.
//! Worlds are stored as blocks and nothing else, so a loaded palace has its
//! boards but no memory of what they said — and since the layout is fixed, the
//! positions are enough to find the names again. It also means the lookup
//! guards itself: a world with no 푯말 blocks in it never matches anything.

use super::*;
use crate::block::Block;
use crate::world::World;
use bevy::math::IVec3;

/// Where each board stands and what it says, as offsets from the precinct
/// centre. Boards sit *beside* the way in rather than on it — a post in the
/// middle of a three-block path is something to walk round.
fn table() -> Vec<(i32, i32, &'static str)> {
    let mut v: Vec<(i32, i32, &'static str)> = Vec::new();
    let mut add = |dx: i32, dz: i32, name: &'static str| v.push((dx, dz, name));

    // The axis, south to north.
    add(s(4), PALACE_SOUTH - s(5), "광화문");
    add(s(4), HEUNGNYE_Z + s(4), "흥례문");
    add(s(4), GEUMCHEON_Z + s(5), "영제교");
    add(s(4), COURT_OFFSET_Z + COURT_Z + s(3), "근정문");
    add(s(11), COURT_OFFSET_Z - s(2) + s(12) + 2, "근정전");
    // The gates through the cross walls, each named for the yard it opens on.
    // 근정문 and the outer gates already have boards; these are the three the
    // axis passes through between the court and the queen's quarters.
    add(s(3), SAJEONG_Z + s(12) - s(5), "사정문");
    add(s(3), GANGNYEONG_Z + s(8) - 2, "향오문");
    add(s(3), GYOTAE_Z + s(8) - 2, "양의문");

    add(s(9), SAJEONG_Z + s(4) + s(3), "사정전");
    add(s(9), GANGNYEONG_Z + s(4) + s(3), "강녕전");
    add(s(9), GYOTAE_Z + s(4) + s(3), "교태전");
    add(s(4), GYOTAE_Z - s(4) - s(5), "아미산");
    add(s(4), HYANGWON_Z + s(9) + 2, "향원정");

    // The flanking halls, in 교태전's own yard.
    add(HAMWON_X - s(4), FLANKING_HALL_Z, "함원전");
    add(HEUMGYEONG_X + s(4), FLANKING_HALL_Z, "흠경각");

    // The compounds either side, each beside its own gateway.
    add(JAGYEONG_X + s(4), JAGYEONG_Z + s(11) + 2, "자경전");
    add(SUJEONG_X + s(4), SUJEONG_Z + s(8) + 2, "수정전");
    add(DONGGUNG_X + s(4), DONGGUNG_Z + DONGGUNG_RZ + 2, "동궁");
    add(HAMHWA_X + s(4), HAMHWA_Z + HAMHWA_RZ + 2, "함화당");
    add(TAEWON_X + s(4), TAEWON_Z + TAEWON_RZ + 2, "태원전");
    add(GEONCHEONG_X - GEONCHEONG_RX - 2, GEONCHEONG_Z + s(3), "건청궁");
    add(JIPOK_X, JIPOK_Z + s(8), "집옥재");
    add(SOJU_X - s(5), SOJU_Z, "소주방");

    // Water, and the wall.
    add(GYEONGHOE_X + s(7) + 2, GYEONGHOE_Z, "경회루");
    add(-PALACE_X + s(3), YEONGCHU_Z + s(4), "영추문");
    add(PALACE_X - s(3), GEONCHUN_Z + s(4), "건춘문");
    add(s(4), -PALACE_NORTH + s(4), "신무문");
    add(PALACE_X - s(5), PALACE_SOUTH - s(6), "동십자각");

    v
}

/// Every board in world coordinates, paired with its name.
pub(crate) fn signposts() -> Vec<(IVec3, &'static str)> {
    let (cx, cz) = (WORLD_X / 2, WORLD_Z / 2);
    table()
        .into_iter()
        .map(|(dx, dz, name)| (IVec3::new(cx + dx, GROUND + 2, cz + dz), name))
        .collect()
}

/// Stand every board up. Runs last, after the paths, so a post can be set on
/// paving without the paving later erasing it.
pub(super) fn place_signposts(world: &mut World, gy: i32) {
    for (pos, _) in signposts() {
        // The post first, then the board on top of it.
        world.set(pos.x, gy + 1, pos.z, Block::Wood);
        world.set(pos.x, pos.y, pos.z, Block::Signpost);
    }
}
