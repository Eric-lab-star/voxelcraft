//! Turn a chunk's blocks into renderable meshes.
//!
//! We build two meshes per chunk:
//!   * **opaque** — every solid, non-water block, built with *greedy meshing*:
//!     coplanar faces that share a tile and ambient-occlusion value are merged
//!     into a single large quad, cutting vertex/triangle counts dramatically on
//!     flat terrain. The atlas is tiled across each merged quad by the terrain
//!     shader (`voxel.wgsl`), which reads the *repeat* UVs and the tile index we
//!     pack into the vertex-colour alpha here.
//!   * **water**  — water blocks, drawn per-face with a translucent
//!     `StandardMaterial`; water is flat, single-tile and cheap, so it keeps the
//!     simple per-face path with atlas UVs baked in.
//!
//! Face culling differs between them so water reads correctly:
//!   * An opaque face shows when its neighbour is *not opaque* (air or water) —
//!     so the lakebed is visible through the water.
//!   * A water face shows only when its neighbour is air — so we don't draw the
//!     submerged sides against terrain or the seams between water blocks.

use crate::block::Block;
use crate::texture::{atlas_uv, block_tile};
use crate::world::{World, CHUNK_SIZE, WORLD_Y};
use bevy::asset::RenderAssetUsages;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};

struct Face {
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
}

const FACES: [Face; 6] = [
    // -X
    Face {
        normal: [-1.0, 0.0, 0.0],
        corners: [[0., 0., 1.], [0., 0., 0.], [0., 1., 0.], [0., 1., 1.]],
    },
    // +X
    Face {
        normal: [1.0, 0.0, 0.0],
        corners: [[1., 0., 0.], [1., 0., 1.], [1., 1., 1.], [1., 1., 0.]],
    },
    // -Y
    Face {
        normal: [0.0, -1.0, 0.0],
        corners: [[0., 0., 1.], [1., 0., 1.], [1., 0., 0.], [0., 0., 0.]],
    },
    // +Y
    Face {
        normal: [0.0, 1.0, 0.0],
        corners: [[0., 1., 0.], [1., 1., 0.], [1., 1., 1.], [0., 1., 1.]],
    },
    // -Z
    Face {
        normal: [0.0, 0.0, -1.0],
        corners: [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
    },
    // +Z
    Face {
        normal: [0.0, 0.0, 1.0],
        corners: [[1., 0., 1.], [0., 0., 1.], [0., 1., 1.], [1., 1., 1.]],
    },
];

const NEIGHBOURS: [[i32; 3]; 6] = [
    [-1, 0, 0],
    [1, 0, 0],
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
];

/// Per-face in-plane layout for greedy meshing: the axis the face points along
/// (`a`), the two in-plane axes mapped to texture U and V, and the sign of the
/// face normal along `a`. Axes are 0 = X, 1 = Y, 2 = Z.
struct FaceAxes {
    a: usize,
    u: usize,
    v: usize,
    sign: i32,
}

const FACE_AXES: [FaceAxes; 6] = [
    // -X: plane in Y/Z, U=Z, V=Y
    FaceAxes { a: 0, u: 2, v: 1, sign: -1 },
    // +X
    FaceAxes { a: 0, u: 2, v: 1, sign: 1 },
    // -Y: plane in X/Z, U=X, V=Z
    FaceAxes { a: 1, u: 0, v: 2, sign: -1 },
    // +Y
    FaceAxes { a: 1, u: 0, v: 2, sign: 1 },
    // -Z: plane in X/Y, U=X, V=Y
    FaceAxes { a: 2, u: 0, v: 1, sign: -1 },
    // +Z
    FaceAxes { a: 2, u: 0, v: 1, sign: 1 },
];

/// Fixed brightness per face direction (Minecraft-style directional shading):
/// tops are brightest, the two horizontal axes differ, bottoms are darkest.
/// This alone makes stacked/terraced terrain readable even from straight above.
const FACE_SHADE: [f32; 6] = [0.65, 0.65, 0.5, 1.0, 0.8, 0.8];

/// Brightness for the four ambient-occlusion levels (0 = deepest corner).
const AO_FACTOR: [f32; 4] = [0.4, 0.62, 0.82, 1.0];

/// A block is *opaque* if it's solid and not water.
fn is_opaque(block: Block) -> bool {
    block.is_solid() && block != Block::Water
}

/// Ambient occlusion for the four corners of one voxel face, in the greedy
/// corner order (u0,v0), (u1,v0), (u1,v1), (u0,v1). Each corner is darkened by
/// how many of its three edge/diagonal neighbours (in the plane just outside the
/// face) are solid — the classic voxel AO that gives blocks soft contact
/// shadows.
fn corner_ao(world: &World, vox: [i32; 3], fa: &FaceAxes) -> [f32; 4] {
    let mut base = vox;
    base[fa.a] += fa.sign;
    // (du, dv) for each corner in (u0,v0),(u1,v0),(u1,v1),(u0,v1) order.
    const DUV: [(i32, i32); 4] = [(-1, -1), (1, -1), (1, 1), (-1, 1)];
    let mut out = [1.0f32; 4];
    for (i, &(du, dv)) in DUV.iter().enumerate() {
        let mut p1 = base;
        p1[fa.u] += du;
        let mut p2 = base;
        p2[fa.v] += dv;
        let mut pc = base;
        pc[fa.u] += du;
        pc[fa.v] += dv;

        let s1 = is_opaque(world.get(p1[0], p1[1], p1[2]));
        let s2 = is_opaque(world.get(p2[0], p2[1], p2[2]));
        let cc = is_opaque(world.get(pc[0], pc[1], pc[2]));

        // Two solid sides fully occlude the corner regardless of the diagonal.
        let level = if s1 && s2 {
            0
        } else {
            3 - (s1 as usize + s2 as usize + cc as usize)
        };
        out[i] = AO_FACTOR[level];
    }
    out
}

/// One merge-able greedy cell: same tile and same (uniform) AO can be combined.
#[derive(Clone, Copy, PartialEq)]
struct Cell {
    tile: u32,
    ao: f32,
}

/// Accumulates geometry for one mesh.
#[derive(Default)]
struct MeshBuf {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuf {
    /// Emit one per-face water quad with atlas UVs baked in.
    ///
    /// Vertex-colour alpha carries a flag the water shader reads: 1 marks a
    /// vertex sitting on the free surface, which the vertex shader displaces
    /// with the wave field; 0 pins it. A vertex is on the surface when it is at
    /// the top of its block (`corner[1] == 1.0`) and that block is `open` to the
    /// air above. Flagging by corner height rather than by face means a top
    /// quad and the upper edge of the side quads beside it get the *same* flag
    /// — and since the displacement is a function of world XZ alone, the
    /// duplicated vertices where they meet move together and the surface stays
    /// watertight.
    fn push_water_face(
        &mut self,
        face: &Face,
        wx: i32,
        wy: i32,
        wz: i32,
        f: usize,
        tile: u32,
        open: bool,
    ) {
        // Side faces list two bottom corners then two top corners, keeping
        // textures upright; top/bottom faces are laid out flat.
        let corner_uv = if f == 2 || f == 3 {
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        } else {
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
        };
        let s = FACE_SHADE[f];
        let start = self.positions.len() as u32;
        for (ci, corner) in face.corners.iter().enumerate() {
            self.positions.push([
                wx as f32 + corner[0],
                wy as f32 + corner[1],
                wz as f32 + corner[2],
            ]);
            self.normals.push(face.normal);
            self.uvs.push(atlas_uv(tile, corner_uv[ci][0], corner_uv[ci][1]));
            let surface = open && corner[1] == 1.0;
            self.colors
                .push([s, s, s, if surface { 1.0 } else { 0.0 }]);
        }
        // Water is drawn without AO, so the diagonal choice is arbitrary; keep
        // the same split the opaque path uses for consistency.
        let ao = [1.0f32; 4];
        if ao[0] + ao[2] > ao[1] + ao[3] {
            self.indices.extend_from_slice(&[
                start,
                start + 1,
                start + 2,
                start,
                start + 2,
                start + 3,
            ]);
        } else {
            self.indices.extend_from_slice(&[
                start + 1,
                start + 2,
                start + 3,
                start + 1,
                start + 3,
                start,
            ]);
        }
    }

    /// Emit one greedy quad for the opaque mesh. The quad spans world voxel
    /// range `[u0,u1) × [v0,v1)` on axis-`a` plane `plane`. UVs are *repeat*
    /// coordinates (one unit per block); the terrain shader turns them into
    /// tiled atlas lookups using the `tile` index we pack into vertex-colour
    /// alpha. `ao` is per corner in (u0,v0),(u1,v0),(u1,v1),(u0,v1) order.
    #[allow(clippy::too_many_arguments)]
    fn push_greedy_quad(
        &mut self,
        fa: &FaceAxes,
        f: usize,
        plane: i32,
        u0: i32,
        u1: i32,
        v0: i32,
        v1: i32,
        tile: u32,
        ao: [f32; 4],
    ) {
        let corner = |uu: i32, vv: i32| -> [f32; 3] {
            let mut p = [0.0f32; 3];
            p[fa.a] = plane as f32;
            p[fa.u] = uu as f32;
            p[fa.v] = vv as f32;
            p
        };
        let c = [
            corner(u0, v0),
            corner(u1, v0),
            corner(u1, v1),
            corner(u0, v1),
        ];

        // Keep textures upright: on the vertical (side) faces V runs along world
        // Y, so flip it to put the tile's top at the block's top. Top/bottom
        // faces (a == 1) are flat, no flip needed.
        let flip_v = fa.a != 1;
        let repeat = |uu: i32, vv: i32| -> [f32; 2] {
            let ur = (uu - u0) as f32;
            let vr = if flip_v {
                (v1 - vv) as f32
            } else {
                (vv - v0) as f32
            };
            [ur, vr]
        };
        let uv = [
            repeat(u0, v0),
            repeat(u1, v0),
            repeat(u1, v1),
            repeat(u0, v1),
        ];

        let normal = FACES[f].normal;
        let s = FACE_SHADE[f];
        let start = self.positions.len() as u32;
        for i in 0..4 {
            self.positions.push(c[i]);
            self.normals.push(normal);
            self.uvs.push(uv[i]);
            let b = s * ao[i];
            self.colors.push([b, b, b, tile as f32]); // tile index packed in alpha
        }

        // Triangulate so the winding matches the intended outward normal (the
        // material is double-sided, so this only keeps front/back lighting
        // consistent). AO here is uniform, so the diagonal choice is irrelevant.
        let e1 = [c[1][0] - c[0][0], c[1][1] - c[0][1], c[1][2] - c[0][2]];
        let e2 = [c[2][0] - c[0][0], c[2][1] - c[0][1], c[2][2] - c[0][2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let facing = cross[0] * normal[0] + cross[1] * normal[1] + cross[2] * normal[2];
        if facing >= 0.0 {
            self.indices
                .extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
        } else {
            self.indices
                .extend_from_slice(&[start, start + 2, start + 1, start, start + 3, start + 2]);
        }
    }

    fn into_mesh(self) -> Option<Mesh> {
        if self.positions.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        Some(mesh)
    }
}

/// The two meshes for a chunk column. Either may be `None` if empty.
pub struct ChunkMeshes {
    pub opaque: Option<Mesh>,
    pub water: Option<Mesh>,
}

pub fn build_chunk_meshes(world: &World, cx: i32, cz: i32) -> ChunkMeshes {
    let mut opaque = MeshBuf::default();
    let mut water = MeshBuf::default();

    let ox = cx * CHUNK_SIZE;
    let oz = cz * CHUNK_SIZE;

    build_opaque(world, &mut opaque, ox, oz);
    build_water(world, &mut water, ox, oz);

    ChunkMeshes {
        opaque: opaque.into_mesh(),
        water: water.into_mesh(),
    }
}

/// Greedy-mesh the opaque blocks of a chunk column, one face direction at a
/// time. Per layer we build a 2-D mask of merge-able cells, emit any face whose
/// AO isn't uniform straight away (as a 1×1 quad, so the AO stays pixel-exact),
/// then merge the remaining flat-AO cells into the largest rectangles possible.
fn build_opaque(world: &World, buf: &mut MeshBuf, ox: i32, oz: i32) {
    // World-space minimum and extent along each axis for this chunk column.
    let axis_min = [ox, 0, oz];
    let axis_ext = [CHUNK_SIZE, WORLD_Y, CHUNK_SIZE];

    for (f, fa) in FACE_AXES.iter().enumerate() {
        let ext_u = axis_ext[fa.u];
        let ext_v = axis_ext[fa.v];
        let ext_a = axis_ext[fa.a];
        let min_u = axis_min[fa.u];
        let min_v = axis_min[fa.v];
        let min_a = axis_min[fa.a];

        let mut mask: Vec<Option<Cell>> = vec![None; (ext_u * ext_v) as usize];

        for la_i in 0..ext_a {
            let la = min_a + la_i;
            // The face sits on the low or high side of the voxel along `a`.
            let plane = la + if fa.sign > 0 { 1 } else { 0 };

            // --- build the mask for this layer --------------------------------
            for iv in 0..ext_v {
                for iu in 0..ext_u {
                    let mut vox = [0i32; 3];
                    vox[fa.a] = la;
                    vox[fa.u] = min_u + iu;
                    vox[fa.v] = min_v + iv;

                    let idx = (iv * ext_u + iu) as usize;
                    let block = world.get(vox[0], vox[1], vox[2]);
                    if !is_opaque(block) {
                        mask[idx] = None;
                        continue;
                    }
                    // Exposed only if the neighbour along the face normal is not
                    // opaque (air or water).
                    let mut nb = vox;
                    nb[fa.a] += fa.sign;
                    if is_opaque(world.get(nb[0], nb[1], nb[2])) {
                        mask[idx] = None;
                        continue;
                    }

                    let tile = block_tile(block, f);
                    let ao = corner_ao(world, vox, fa);
                    if ao[0] == ao[1] && ao[1] == ao[2] && ao[2] == ao[3] {
                        mask[idx] = Some(Cell { tile, ao: ao[0] });
                    } else {
                        // Non-uniform AO can't be stretched across a merge
                        // without artifacts — emit it now as a single face.
                        let u0 = min_u + iu;
                        let v0 = min_v + iv;
                        buf.push_greedy_quad(fa, f, plane, u0, u0 + 1, v0, v0 + 1, tile, ao);
                        mask[idx] = None;
                    }
                }
            }

            // --- greedy-merge the remaining flat-AO cells ---------------------
            merge_mask(buf, &mut mask, ext_u, ext_v, fa, f, plane, min_u, min_v);
        }
    }
}

/// Classic 2-D greedy merge over a mask of merge-able cells: grow each run as
/// wide as possible, then as tall as possible, emit one quad, and clear it.
#[allow(clippy::too_many_arguments)]
fn merge_mask(
    buf: &mut MeshBuf,
    mask: &mut [Option<Cell>],
    ext_u: i32,
    ext_v: i32,
    fa: &FaceAxes,
    f: usize,
    plane: i32,
    min_u: i32,
    min_v: i32,
) {
    for iv in 0..ext_v {
        let mut iu = 0;
        while iu < ext_u {
            let idx = (iv * ext_u + iu) as usize;
            let Some(cell) = mask[idx] else {
                iu += 1;
                continue;
            };

            // Grow width along U.
            let mut w = 1;
            while iu + w < ext_u && mask[(iv * ext_u + iu + w) as usize] == Some(cell) {
                w += 1;
            }
            // Grow height along V, one full row at a time.
            let mut h = 1;
            'grow: while iv + h < ext_v {
                for k in 0..w {
                    if mask[((iv + h) * ext_u + iu + k) as usize] != Some(cell) {
                        break 'grow;
                    }
                }
                h += 1;
            }

            let u0 = min_u + iu;
            let v0 = min_v + iv;
            buf.push_greedy_quad(fa, f, plane, u0, u0 + w, v0, v0 + h, cell.tile, [cell.ao; 4]);

            // Consume the merged rectangle.
            for dv in 0..h {
                for du in 0..w {
                    mask[((iv + dv) * ext_u + iu + du) as usize] = None;
                }
            }
            iu += w;
        }
    }
}

/// Build the (per-face, un-merged) water surface mesh for a chunk column.
fn build_water(world: &World, buf: &mut MeshBuf, ox: i32, oz: i32) {
    for ly in 0..WORLD_Y {
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let (wx, wy, wz) = (ox + lx, ly, oz + lz);
                if world.get(wx, wy, wz) != Block::Water {
                    continue;
                }
                // Open to the sky, so this block's top edge is the free surface
                // the waves displace. A submerged block keeps its corners pinned
                // even where a side face is exposed (a waterfall wall), or it
                // would tear away from the block stacked on it.
                let open = world.get(wx, wy + 1, wz) != Block::Water;
                for (f, face) in FACES.iter().enumerate() {
                    let n = NEIGHBOURS[f];
                    // A water face shows only against air.
                    if world.get(wx + n[0], wy + n[1], wz + n[2]) != Block::Air {
                        continue;
                    }
                    let tile = block_tile(Block::Water, f);
                    buf.push_water_face(face, wx, wy, wz, f, tile, open);
                }
            }
        }
    }
}
