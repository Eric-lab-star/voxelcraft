//! Turn a chunk's blocks into renderable meshes.
//!
//! We build two meshes per chunk:
//!   * **opaque** — every solid, non-water block.
//!   * **water**  — water blocks, drawn with a translucent material.
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

/// Fixed brightness per face direction (Minecraft-style directional shading):
/// tops are brightest, the two horizontal axes differ, bottoms are darkest.
/// This alone makes stacked/terraced terrain readable even from straight above.
const FACE_SHADE: [f32; 6] = [0.65, 0.65, 0.5, 1.0, 0.8, 0.8];

/// Brightness for the four ambient-occlusion levels (0 = deepest corner).
const AO_FACTOR: [f32; 4] = [0.4, 0.62, 0.82, 1.0];

/// Per-vertex ambient occlusion for a face: each of the four corners is darkened
/// by how many of its three diagonally/edge-adjacent neighbours are solid.
/// This is the classic voxel AO that gives blocks soft contact shadows.
fn face_ao(world: &World, wx: i32, wy: i32, wz: i32, f: usize) -> [f32; 4] {
    let n = NEIGHBOURS[f];
    let base = [wx + n[0], wy + n[1], wz + n[2]];
    // Which axis the face points along, and the two in-plane (tangent) axes.
    let a = if n[0] != 0 {
        0
    } else if n[1] != 0 {
        1
    } else {
        2
    };
    let (t1, t2) = match a {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };

    let corners = FACES[f].corners;
    let mut out = [1.0f32; 4];
    for ci in 0..4 {
        let c = corners[ci];
        let d1 = if c[t1] > 0.5 { 1 } else { -1 };
        let d2 = if c[t2] > 0.5 { 1 } else { -1 };

        let mut p_side1 = base;
        p_side1[t1] += d1;
        let mut p_side2 = base;
        p_side2[t2] += d2;
        let mut p_corner = base;
        p_corner[t1] += d1;
        p_corner[t2] += d2;

        let s1 = is_opaque(world.get(p_side1[0], p_side1[1], p_side1[2]));
        let s2 = is_opaque(world.get(p_side2[0], p_side2[1], p_side2[2]));
        let cc = is_opaque(world.get(p_corner[0], p_corner[1], p_corner[2]));

        // Two solid sides fully occlude the corner regardless of the diagonal.
        let level = if s1 && s2 {
            0
        } else {
            3 - (s1 as usize + s2 as usize + cc as usize)
        };
        out[ci] = AO_FACTOR[level];
    }
    out
}

/// A block is *opaque* if it's solid and not water.
fn is_opaque(block: Block) -> bool {
    block.is_solid() && block != Block::Water
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
    fn push_face(&mut self, face: &Face, wx: i32, wy: i32, wz: i32, f: usize, tile: u32, ao: [f32; 4]) {
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
            let b = s * ao[ci]; // face shading × ambient occlusion
            self.colors.push([b, b, b, 1.0]);
        }
        // Split the quad along the diagonal that keeps AO interpolation
        // symmetric (avoids the classic voxel-AO seam artifact).
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

    for ly in 0..WORLD_Y {
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let (wx, wy, wz) = (ox + lx, ly, oz + lz);
                let block = world.get(wx, wy, wz);
                if block == Block::Air {
                    continue;
                }
                let is_water = block == Block::Water;

                for (f, face) in FACES.iter().enumerate() {
                    let n = NEIGHBOURS[f];
                    let neighbour = world.get(wx + n[0], wy + n[1], wz + n[2]);

                    let visible = if is_water {
                        neighbour == Block::Air
                    } else {
                        !is_opaque(neighbour)
                    };
                    if !visible {
                        continue;
                    }

                    let tile = block_tile(block, f);
                    // Water is a flat surface — skip AO; opaque blocks get it.
                    let ao = if is_water {
                        [1.0; 4]
                    } else {
                        face_ao(world, wx, wy, wz, f)
                    };
                    let buf = if is_water { &mut water } else { &mut opaque };
                    buf.push_face(face, wx, wy, wz, f, tile, ao);
                }
            }
        }
    }

    ChunkMeshes {
        opaque: opaque.into_mesh(),
        water: water.into_mesh(),
    }
}
