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
//!                        향원정   (hexagonal pavilion, on its island)   건청궁 집옥재
//!                        아미산   (terraced garden)
//!              함원전   교태전   흠경각 ┐ 무량각 — the king's and
//!    영추문   수정전     강녕전          ┘ queen's halls have no ridge  자경전
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
// The palace is one design split across several files, not a set of
// independent components: a hall needs the roof builder, the roof builder
// needs the shared proportions, the paths need to know where every compound's
// gateway is. So each file pulls in the whole vocabulary with `use super::*`
// and the re-exports below carry the pieces between them.
#[cfg(test)]
mod checks;
mod compound;
mod gate;
mod hall;
mod path;
mod sign;
mod style;
mod water;

use compound::*;
use gate::*;
use hall::*;
use path::*;
pub(crate) use sign::signposts;
use sign::place_signposts;
use style::*;
use water::*;


use crate::world::{FLAT_LEVEL, WORLD_X, WORLD_Z, World};

/// The level everything is built on. Shared with the blank map so the two flat
/// worlds sit at the same height.
const GROUND: i32 = FLAT_LEVEL;

pub fn generate(_seed: u32) -> World {
    let mut world = World::empty();
    world.fill_flat(GROUND);
    place_palace(&mut world, GROUND);
    world
}

// --- 경복궁 (the palace) ----------------------------------------------------

/// Half-extents of the walled palace precinct.
///
/// Widened from 38 once the interior ran out of room. The precinct is laid out
/// in bands either side of the axis — halls, then a strip for the small halls
/// beside them, then the flanking route, the inner yard wall, a service strip,
/// and the side compounds — and at the old width those bands were four and nine
/// blocks across, too thin to stand anything in. Widening them all pushed this
/// out with them.
///
/// It is more faithful as well as more usable. 경복궁 is about 500m by 700m,
/// or 1:1.40; at 38 this precinct was 1:1.72, noticeably narrower than the real
/// thing. 48 puts it at 1:1.35.
const PALACE_X: i32 = d(48);

/// How far the precinct runs south and north of its centre. Gyeongbokgung is far
/// deeper than it is wide, and lopsided about its middle: the ceremonial gate and
/// court sit at the south end, and the halls the royal family actually lived in
/// run away north behind them.
const PALACE_SOUTH: i32 = d(30);

/// The north wall used to stand two blocks off 향원정's pond, which left no
/// room at all for 신무문 in it. Carried further out instead of squeezing the
/// pond: the real palace has a whole quarter up here, and this is where it
/// would go.
const PALACE_NORTH: i32 = d(100);

/// Half-extents of the 근정전 court — the cloistered inner yard the throne hall
/// stands in. A throne hall alone in an open field reads as a big shed; the
/// enclosure is what makes it the centre of a palace.
const COURT_X: i32 = d(25);
const COURT_Z: i32 = d(15);

/// How far north of the precinct centre that court sits, leaving a long
/// approach between 광화문 and its gate.
const COURT_OFFSET_Z: i32 = -d(13);

/// Centres of the halls behind the throne hall, north of the court, as offsets
/// from the precinct centre. Each stands in its own walled yard.
const SAJEONG_Z: i32 = -d(38); // 사정전, where the king held council
const GANGNYEONG_Z: i32 = -d(52); // 강녕전, the king's own quarters
const GYOTAE_Z: i32 = -d(66); // 교태전, the queen's
/// 향원정, in the rear garden well beyond the living quarters.
const HYANGWON_Z: i32 = -d(84);

/// 함원전 and 흠경각, in the strips either side of 교태전.
///
/// These are what widening the precinct was for. The strip is 15 blocks between
/// the queen's platform and the flanking route, and a hall of this size lands
/// two clear of each — and two clear of 아미산's lowest terrace behind.
///
/// Small, and correctly so. Neither was a hall of state: 흠경각 housed the
/// astronomical clock and 함원전 was a private chapel, so at 침전 scale they
/// would tower over the queen's own quarters standing between them.
const HAMWON_X: i32 = -d(14);
const HEUMGYEONG_X: i32 = d(14);
const FLANKING_HALL_Z: i32 = -d(65);

/// 건청궁, in the quarter north-east of the pond. Carrying the north wall out
/// to make room for 신무문 left this whole corner empty, and it is where the
/// real 건청궁 stands: off the axis, behind the garden, away from the halls of
/// state entirely — which is the point of it. It was built as somewhere the
/// king could live outside the ceremonial palace.
///
/// The corner is tighter than it looks. 자경전's yard reaches up to z -94 on
/// this flank and the garden path to 향원정 runs north at s(16) east of the
/// axis, so this compound sits between the two of them. The precinct has since
/// been widened, which is what opened the ground east of it.
/// 집옥재, in the ground east of 건청궁 that widening the precinct opened up.
const JIPOK_X: i32 = d(42);
const JIPOK_Z: i32 = -d(81);

const GEONCHEONG_X: i32 = d(27);
const GEONCHEONG_Z: i32 = -d(81);
const GEONCHEONG_RX: i32 = d(9);
const GEONCHEONG_RZ: i32 = d(16);

/// 장안당, the king's side, in front of 곤녕합, the queen's.
const JANGAN_Z: i32 = -d(8);
const GONNYEONG_Z: i32 = d(8);

/// 태원전, in the north-west quarter, between the west wall and the flanking
/// path and running 83 deep from the north wall down to 수정전.
const TAEWON_X: i32 = -d(34);
const TAEWON_Z: i32 = -d(76);
const TAEWON_RX: i32 = d(11);
const TAEWON_RZ: i32 = d(14);

/// The shrine hall stands deep in its yard, at the back of the court.
const TAEWON_HALL_Z: i32 = -d(6);

/// 함화당 and 집경당, in the west-central ground between 태원전 and 수정전.
///
/// Went in when this was the last open ground with room for a compound. The
/// precinct has since been widened, so it is no longer the last.
const HAMHWA_X: i32 = -d(34);
const HAMHWA_Z: i32 = -d(53);
const HAMHWA_RX: i32 = d(11);
const HAMHWA_RZ: i32 = d(7);
/// The two halls stand side by side, the ground here being wider than deep.
///
/// How far apart is fixed rather than chosen: the yard is 33 blocks across and
/// each hall's roof is 13 of them, so there are 7 to share out — two to each
/// wall and three between the pair. At `s(5)` they sat a single block apart and
/// read as one building with a seam down it.
const HAMHWA_SPREAD: i32 = 8;

/// The side compounds, in the outermost band — beyond the inner yard wall at
/// 34 and inside the precinct wall at 72. Their half-width is 10, so a centre
/// of 58 spans 48..68, leaving thirteen blocks of service strip inside them and
/// four to the wall outside.
const JAGYEONG_X: i32 = d(39); // 자경전, the dowager queen's hall
const JAGYEONG_Z: i32 = -d(52);

const SUJEONG_X: i32 = -d(39); // 수정전, west of the axis
const SUJEONG_Z: i32 = -d(36);

const DONGGUNG_X: i32 = d(39); // 동궁, the crown prince's quarters
/// 동궁's compound. Its band is 21 blocks across, so its two halls cannot stand
/// side by side however the real ones are drawn — the yard runs deep instead
/// and puts one behind the other.
const DONGGUNG_RX: i32 = d(7);

const DONGGUNG_RZ: i32 = d(15);

/// 자선당, where the crown prince lived, in front of 비현각, where he was
/// taught. Offsets from the compound's centre.
const JASEON_Z: i32 = -d(8);

const BIHYEON_Z: i32 = d(8);

/// 동궁 and 자경전 share the east flank, and at -34 their compound walls
/// overlapped by two blocks — the two yards ran into one another with no gap
/// between. Moved south far enough to separate them and to leave a gap in the
/// east wall wide enough for 건춘문.
const DONGGUNG_Z: i32 = -d(18);

/// Half-extents of the throne hall's two 월대 terraces.
const WOLDAE_OUTER: (i32, i32) = (s(15), s(12));

const WOLDAE_INNER: (i32, i32) = (s(12), s(9));

/// Where the palace stands in the world.
///
/// Centred east-west but set well south of the middle, which is both what the
/// ground wants and what the real city looks like: 북악산 rises immediately
/// behind 경복궁 and 육조거리 runs away south from 광화문, so the north needs
/// depth for a mountain and the south for a street. Dead centre the precinct
/// left too little of either — at this scale it would not fit at all.
pub(crate) fn palace_centre() -> (i32, i32) {
    (WORLD_X / 2, WORLD_Z * 5 / 8)
}

/// Build 경복궁: a walled precinct entered from the
/// south through 광화문, with 근정전 raised on its 월대 terraces at the north end
/// and a stone-paved court between them.
fn place_palace(world: &mut World, gy: i32) {
    let (cx, cz) = palace_centre();
    lay_courtyard(world, cx, cz, gy);
    build_wall(world, cx, cz, gy);
    place_gate(world, cx, cz + PALACE_SOUTH, gy);
    // The three lesser gates, so the precinct is not a walled box with one door.
    place_wall_gate(world, cx - PALACE_X, cz + YEONGCHU_Z, gy, false); // 영추문
    place_wall_gate(world, cx + PALACE_X, cz + GEONCHUN_Z, gy, false); // 건춘문
    place_wall_gate(world, cx, cz - PALACE_NORTH, gy, true); // 신무문
    // 동십자각, on the corner the wall turns at.
    place_corner_tower(world, cx + PALACE_X, cz + PALACE_SOUTH, gy);

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
    place_gyeonghoeru(world, cx + GYEONGHOE_X, cz + GYEONGHOE_Z, gy);
    // 자경전 — the dowager queen's hall, in the matching strip to the east.
    place_jagyeongjeon(world, cx + JAGYEONG_X, cz + JAGYEONG_Z, gy);
    // 수정전 and 동궁 fill the flanks either side of the inner yards, which were
    // bare ground between the cloister and the precinct wall.
    compound_wall(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, s(7), s(8), false);
    place_residence(world, cx + SUJEONG_X, cz + SUJEONG_Z, gy, s(5), s(4), true);
    place_donggung(world, cx + DONGGUNG_X, cz + DONGGUNG_Z, gy);
    // 향원정 — the hexagonal pavilion in the rear garden, at the far north.
    place_hyangwonjeong(world, cx, cz + HYANGWON_Z, gy);
    // 건청궁, in the corner beyond it.
    place_geoncheongung(world, cx + GEONCHEONG_X, cz + GEONCHEONG_Z, gy);
    // 집옥재, the library, in the corner beyond 건청궁.
    place_jipokjae(world, cx + JIPOK_X, cz + JIPOK_Z, gy);
    // 태원전, in the matching corner to the west.
    place_taewonjeon(world, cx + TAEWON_X, cz + TAEWON_Z, gy);
    // 함원전 and 흠경각, either side of 교태전 in its own yard.
    place_residence(world, cx + HAMWON_X, cz + FLANKING_HALL_Z, gy, s(2), s(2), true);
    place_residence(world, cx + HEUMGYEONG_X, cz + FLANKING_HALL_Z, gy, s(2), s(2), true);
    // 함화당 and 집경당, in the west-central ground.
    place_hamhwadang(world, cx + HAMHWA_X, cz + HAMHWA_Z, gy);
    // 소주방, in the service strip behind 강녕전.
    place_sojubang(world, cx + SOJU_X, cz + SOJU_Z, gy);

    // Last, so every gateway it has to meet is already standing. Paths are laid
    // at ground level only, so this cannot disturb any of them.
    lay_paths(world, cx, cz, gy);
    // The 푯말 last of all, so nothing laid afterwards can bury one.
    place_signposts(world, gy);
}

// --- 궁문 (the secondary gates in the precinct wall) ------------------------

/// Where 영추문, 건춘문 and 신무문 stand, as offsets from the precinct centre.
/// The two side gates sit in the gaps between the compounds along each flank —
/// 수정전 and 경회루's pond on the west, 자경전 and 동궁 on the east.
const YEONGCHU_Z: i32 = -d(24);

const GEONCHUN_Z: i32 = -d(38);

// --- 소주방 (the kitchens) ---------------------------------------------------

/// 소주방, in the strip east of 강녕전 — thirteen blocks of clear ground between
/// the flanking path and 자경전's yard.
/// 경회루, on its pond in the strip west of the court. Its position was
/// repeated as a bare `s(30)` in three places — the placement, the path along
/// its bank, and the test that has to reach it — which is exactly the kind of
/// thing that comes apart the moment the precinct is re-proportioned.
const GYEONGHOE_X: i32 = -d(36);
const GYEONGHOE_Z: i32 = -d(8);

const SOJU_X: i32 = d(27);

const SOJU_Z: i32 = -d(52);

// --- 흥례문 권역 (the outer approach) ---------------------------------------

/// 흥례문, midway up the approach, and 금천 with 영제교 over it just inside.
const HEUNGNYE_Z: i32 = d(20);

const GEUMCHEON_Z: i32 = d(12);
