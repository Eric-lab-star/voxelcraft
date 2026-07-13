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
    let local = fract(in.uv);           // [0,1] within the current tile
    let u = (col + inset + local.x * (1.0 - 2.0 * inset)) / cols;
    let v = (row + inset + local.y * (1.0 - 2.0 * inset)) / rows;
    in.uv = vec2<f32>(u, v);
    // Drop the packed tile index so it can't tint/blend the shaded colour.
    in.color = vec4<f32>(in.color.rgb, 1.0);
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
