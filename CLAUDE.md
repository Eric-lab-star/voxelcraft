# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

voxelcraft is a Bevy 0.19 voxel sandbox (Rust 2024 edition). The original meadow
map is a small procedural terrain; the bulk of recent work is the **조선 map**
(`src/joseon/`), a to-scale reconstruction of 경복궁 (Gyeongbokgung palace)
built entirely from voxel-placement code.

## Commands

```sh
cargo run --release          # run the game (release — debug framerate is poor)
cargo test --quiet           # all tests (mostly palace-layout invariants)
cargo test --quiet <name>    # a single test by substring, e.g. cargo test --quiet the_wall_holds
cargo test --release --quiet <name> -- --nocapture   # a probe test that prints, at full speed
cargo clippy --quiet         # lint — baseline is ~10 pre-existing warnings; keep it there
```

Environment is Windows with Git Bash. Prefer `rg` over `grep`. `cargo run` never
exits on its own — run it under `timeout 40 cargo run --release` and grep the
output for `error`/`panic` to confirm a change loads (exit code 143 is the
timeout killing it, which is success). The world is built behind a title screen,
so a shader or worldgen panic shows up in that startup log, not before it.

## Architecture

**World storage** (`world.rs`): a flat `Vec<Block>` plus a parallel `Vec<u8>` of
water levels — no chunks in the data model, only in the mesh. `WORLD_X/Y/Z` are
compile-time constants (currently 768×128×768); changing them invalidates every
existing save (the `VOX2` header stores dimensions and `load` rejects a
mismatch). Saves are run-length encoded `(u16 count, u8 id)` pairs — these worlds
are almost all air, so a slot is ~220KB not ~72MB. `find_spawn` tries the palace
gate first (see below) and falls back to a grass search for other maps.

**Meshing → shader atlas tiling** (`mesh.rs`, `voxel.wgsl`, `texture.rs`): the
terrain is greedy-meshed per 16-wide column, merging equal block faces into big
quads. A single atlas tile can't stretch across a merged quad, so the mesh emits
*world-space* repeat UVs (one unit = one block) and packs the atlas tile index
into vertex-colour alpha; the shader recovers the per-tile coord with `fract()`
and the per-*block* cell with `floor()`. Two consequences worth knowing:
- Greedy meshing only merges faces showing the **same tile**, so you cannot give
  neighbouring blocks different tiles without shattering the merge. Per-block
  visual variety (mirror + brightness jitter) is done in the shader from the
  world cell, not by adding tiles.
- All camera setups sharing the window **must use the same MSAA** (`Msaa::Off`)
  or the second camera blanks out. See `chunk.rs` / `main.rs`.

Every texture is painted in code (`texture.rs`, 16×16 tiles). Tiles that cover a
continuous surface (roof, granite) must have their seam period divide 16 or a
doubled dark band appears where copies meet — there is a test for this.

**Procedural palace** (`src/joseon/`) is the largest and most invariant-heavy
subsystem. `generate()` fills a flat plain then places buildings by writing
blocks directly. Files are split by *what a thing is*, and every file does
`use super::*` because the palace is one design, not independent components:

| file | contents |
|------|----------|
| `mod.rs` | layout position constants, `place_palace`, `palace_centre`, `approach_spawn` |
| `style.rs` | `s`/`d` scale fns, shared proportions, `lay_roof`, `lay_brackets` |
| `hall.rs` | halls on platforms, 근정전, the 침전, 아미산 |
| `compound.rs` | walled yards (자경전, 동궁, 태원전, 건청궁, 함화당, 소주방), 집옥재 |
| `gate.rs` | 광화문, timber gates, wall gates, 동십자각, the precinct wall |
| `water.rs` | 경회루, 향원정, 금천 |
| `path.rs` | the 삼도/어도, flanking routes, cloister |
| `sign.rs` | 푯말 name boards + the position→name table |
| `checks.rs` | tests (see below) |

### Palace conventions that are easy to get wrong

- **Two scale functions, and they are not interchangeable.** `s(n)` scales a
  *building* (a hall's half-extents, a wall's height, an eave's reach). `d(n)`
  scales a *distance* (where something stands, how wide a yard is). They were
  deliberately pulled apart because the buildings were near life-size while the
  spacing was 3–4× too tight. A distance written with `s` is a latent bug that
  surfaces the next time the scales move — audit for `cx ± s(...)` /
  `cz ± s(...)` in position arithmetic.
- **Derive, don't restate.** A compound's yard is computed from the halls it
  holds (`yard_for`), not stated as an independent number. Repeating a position
  as a bare literal in several call sites (this happened to 경회루 and to the
  side compounds) is exactly what breaks under a rescale. If a value is used in
  more than one place, make it a named constant and reference it.
- **Paths are laid last, at ground level only.** Walls foot at `gy+1`, so a path
  can only replace the ground *under* a structure — invisible beneath a wall,
  showing through at a gateway. Compound gateways are in the *south* face, so a
  spur approaches from below and turns up into the gate, never broadside.
- Everything is positioned relative to `palace_centre()` (south of world centre,
  to leave room for 북악산 north and 육조거리 south). Do not assume map centre.

### Testing style — measure, don't compute

The recurring failure mode in this codebase is trusting hand-arithmetic about a
dense layout. The reliable workflow, used throughout the git history:

1. Add a temporary `probe_*` test to `checks.rs` that prints a plan view, a
   section, or measured clearances; run it with `--release --nocapture`.
2. Read the actual geometry, fix the real numbers, then **delete the probe**.
3. For any invariant a change could silently break, add a permanent test that
   asserts the *property* (not specific coordinates) and confirm it **fails**
   with the bug reintroduced before trusting it.

Existing permanent tests guard the invariants that have actually broken:
`every_hall_is_on_the_path_network` and `the_halls_can_be_passed_without_...`
(reachability on *paved, standable* ground, not open grass), `the_wall_holds_...`
(no way in except through a gate), `no_two_compounds_overlap`,
`every_signpost_stands_clear`, `a_save_round_trips_exactly`. A pure refactor of
the palace can be verified by fingerprinting the generated world before and
after (hash every block) — it must be identical.

## Commit conventions

Commits here are small and single-purpose, with bodies that explain *why* and
record the mistakes a change made and how a test caught them (see `git log`).
Match that. End commit messages with the `Co-Authored-By` trailer.
