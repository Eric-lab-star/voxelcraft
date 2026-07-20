# voxelcraft

A small Minecraft-like voxel sandbox written in **Rust** with the [Bevy](https://bevyengine.org/) game engine.

Walk around a procedurally generated block world, dig and build, watch the sun
set, and shape the terrain — all rendered with hand-made procedural textures and
no external assets.

## Features

- **Procedural world** — 256×256 block terrain with rolling hills, lakes,
  beaches and scattered trees, generated from layered Perlin noise.
- **First-person player** — walking, jumping and axis-by-axis AABB collision
  against the voxel grid. Invisible walls (inset from the edge) keep you in
  bounds while distant terrain stays visible.
- **Build & mine** — break blocks (with a burst of particles) and place any of 7
  block types. A wireframe highlight shows the targeted block. You can even
  aim through water to build on the lakebed underwater.
- **Chunked meshing** — the world is meshed per 16×16 column with hidden-face
  culling, Minecraft-style per-face directional shading, and smooth per-vertex
  **ambient occlusion**.
- **Procedural textures** — every block texture, the cloud layer, and the window
  icon are painted in code (no image files), sampled nearest-neighbour for a
  crisp pixel look.
- **Flowing water** — a cellular-automaton water simulation: water falls, spreads
  thinly across flat ground, and drains into gaps. Water is translucent, and
  going under the surface tints the screen blue with rising bubbles.
- **Day/night cycle** — the sun orbits over ~10 minutes; sky colour, sunlight,
  ambient light and distance fog all shift from day → sunset → night.
- **Sky** — drifting, procedurally-generated clouds and a distance-fog horizon
  that makes the world read as endless.
- **Hotbar** — pick the active block with number keys `1`–`7` or the mouse wheel.
- **Pause menu & saves** — `Tab` opens a menu with 3 save slots, load, and quit;
  an on-screen toast confirms saves. `F5`/`F9` quick-save/load slot 1.

## Controls

| Input | Action |
|-------|--------|
| Mouse | Look |
| `W` `A` `S` `D` | Move (`Ctrl` = sprint) |
| `Space` | Jump (hold to keep hopping) |
| `1`–`7` / wheel | Select hotbar block |
| Left click | Break block |
| Right click | Place selected block |
| `Tab` | Pause menu (save / load / quit) |
| `F5` / `F9` | Quick save / load (slot 1) |
| `Esc` | Release / recapture the mouse cursor |

## Building & running

Requires a Rust toolchain (2024 edition).

```sh
cargo run            # debug
cargo run --release  # smoother framerate
```

The first build compiles Bevy and its dependencies, so it takes a few minutes;
subsequent builds are fast.

## Windows bundle (distribution)

To produce a shareable Windows build, run the packaging script from the project
root:

```powershell
powershell -ExecutionPolicy Bypass -File package-windows.ps1
```

It builds the release binary, stages a runnable folder, and zips it:

- `dist/voxelcraft-windows/` — the runnable folder
- `dist/voxelcraft-windows-x64.zip` — zipped for sharing (~27 MB)

voxelcraft has **no external asset files** (every texture, the cloud layer and
the window icon are generated in code), so the bundle is just `voxelcraft.exe`
plus a short `PLAY-ME.txt` and this README — no installer, no data folder.

To play, unzip and double-click `voxelcraft.exe`. Worlds are saved next to the
`.exe` as `world_1.sav`..`world_3.sav`. Requires a 64-bit Windows PC with a GPU
that supports Vulkan or DirectX 12 (any reasonably modern machine).

## Project structure

All logic lives in `src/`, one module per concern:

| Module | Responsibility |
|--------|----------------|
| `main.rs` | App setup, plugin/resource/system registration |
| `world.rs` | Voxel grid, terrain generation, water levels, save/load serialization |
| `block.rs` | Block types and their properties |
| `chunk.rs` | Per-chunk render entities, mesh rebuilds, materials, camera + fog |
| `mesh.rs` | Chunk meshing: face culling, ambient occlusion, per-face shading, UVs |
| `texture.rs` | Procedural texture atlas, cloud texture, window icon |
| `player.rs` | Walking controller: gravity, collision, mouse look |
| `interaction.rs` | Voxel raycasting, breaking/placing, target highlight |
| `hotbar.rs` | Hotbar UI and block selection |
| `water.rs` | Water flow simulation + underwater tint and bubbles |
| `particles.rs` | Block-break particles |
| `daynight.rs` | Day/night cycle (sun, sky, ambient, fog) |
| `clouds.rs` | Drifting cloud layer |
| `menu.rs` | Pause menu, save slots, toast messages |
| `save.rs` | Save/load slots on disk |

## Implementation notes

- **World data** is a flat `Vec<Block>` plus a parallel `Vec<u8>` of water
  levels; it's saved to `world_N.sav` as a magic header + one byte per block.
- **Water** uses Minecraft-style levels (a source is level 8; flowing water
  decreases one step per block and stops after 4 blocks), stepped on a timer via
  an active-cell set so idle water costs nothing.
- **Rendering** is stock Bevy PBR: one `StandardMaterial` per texture atlas, with
  vertex colours carrying face shading × ambient occlusion.

## Tech

Rust · Bevy 0.19 · `noise` (Perlin terrain) · `winit` (window icon).

## Credits

UI text is set in **[Galmuri11](https://galmuri.quiple.dev/)** by Lee Minseo, a
Hangul pixel font used under the [SIL Open Font License 1.1](assets/fonts/Galmuri-OFL.txt).
Bevy's bundled default font covers only Latin, so Korean rendered as blank space
without it. The font is compiled into the binary; the licence text ships
alongside the executable.

Everything else — every block texture, the cloud layer, the window icon — is
generated procedurally in code.

---

*Built as a from-scratch learning project — a walkable, buildable voxel world in
a few hundred lines of Rust per system.*
