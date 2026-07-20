// Fragment shader for the greedy-meshed opaque terrain.
//
// Greedy meshing merges many equal block faces into one large quad. A single
// atlas tile can't just be stretched across that quad — it would smear one
// 16×16 texture over many blocks. Instead the mesh feeds us *repeat*
// coordinates in `in.uv` (one unit == one block) and packs the atlas tile index
// into the vertex-colour alpha. Here we recover the per-tile [0,1] coordinate
// with `fract()` and map it into the correct atlas cell, so the texture tiles
// once per block exactly as it did with per-face meshing. Everything else
// (lighting, ambient occlusion baked into vertex-colour RGB, distance fog) is
// delegated to the standard PBR path.

#import bevy_pbr::{
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_fragment::pbr_input_from_standard_material,
    decal::clustered::apply_decals,
    forward_io::{VertexOutput, FragmentOutput},
}

#ifdef VISIBILITY_RANGE_DITHER
#import bevy_pbr::pbr_functions::visibility_range_dither
#endif

// x = atlas columns, y = atlas rows, z = half-texel inset, w = unused.
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> atlas_dims: vec4<f32>;

// A stable value hash of a block cell -> [0,1). Mirrors `texture::hash` on the
// CPU side so the two agree about what a given cell looks like.
fn cell_hash(cell: vec2<f32>) -> f32 {
    // Done entirely in u32: the mix relies on multiplication wrapping, and
    // signed overflow is not something to lean on. `cell` is already integral
    // (it comes from `floor`), so the bitcast is exact for negative cells too —
    // and they do occur, since V runs *down* from world Y on side faces.
    let p = vec2<i32>(cell);
    var h = bitcast<u32>(p.x) * 374761393u + bitcast<u32>(p.y) * 668265263u;
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
    return f32(h % 1000u) / 1000.0;
}

@fragment
fn fragment(
    vertex_output: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var in = vertex_output;

#ifdef VISIBILITY_RANGE_DITHER
    visibility_range_dither(in.position, in.visibility_range_dither);
#endif

    // --- Greedy-mesh atlas tiling -----------------------------------------
    let cols = atlas_dims.x;
    let rows = atlas_dims.y;
    let inset = atlas_dims.z;
    let tile = round(in.color.a);       // atlas tile index packed in alpha
    let col = tile % cols;
    let row = floor(tile / cols);
    let cell = floor(in.uv);            // which block, in world coordinates
    var local = fract(in.uv);           // [0,1] within the current tile

    // Mirror the tile left-to-right on half the blocks. One 16x16 tile stamped
    // identically across a wall reads as a grid from any distance, and the
    // bigger the building the worse it gets — the palace at its new scale is
    // wall after wall of the same stone. Mirroring costs nothing, needs no
    // extra quads (which per-block *tile* variety would, since greedy meshing
    // can only merge faces showing the same tile), and breaks the grid up
    // because the seam between two blocks stops being a repeat.
    //
    // Only the U axis. Every tile here is drawn with a definite top — grass
    // above dirt, the lip of a roof course, the lattice of a 창호 — so a
    // vertical flip would turn them upside down.
    let mirror = cell_hash(cell) < 0.5;
    if mirror {
        local.x = 1.0 - local.x;
    }

    let u = (col + inset + local.x * (1.0 - 2.0 * inset)) / cols;
    let v = (row + inset + local.y * (1.0 - 2.0 * inset)) / rows;
    in.uv = vec2<f32>(u, v);

    // A little per-block brightness jitter on top. The mirror breaks up the
    // pattern; this breaks up the flatness, so a large plastered wall or a
    // granite terrace stops reading as one poster-flat surface. Kept small —
    // past a few percent it stops looking like stone and starts looking like
    // the blocks are lit differently.
    let jitter = 1.0 + (cell_hash(cell + 17.0) - 0.5) * 0.09;
    in.color = vec4<f32>(in.color.rgb * jitter, 1.0);
    // ----------------------------------------------------------------------

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color =
        alpha_discard(pbr_input.material, pbr_input.material.base_color);
    apply_decals(&pbr_input);

    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) == 0u {
        out.color = apply_pbr_lighting(pbr_input);
    } else {
        out.color = pbr_input.material.base_color;
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
