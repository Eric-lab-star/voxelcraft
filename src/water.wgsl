// Vertex shader for the water surface: a sum of Gerstner waves that actually
// displaces the mesh, instead of the flat quads the tile animation used to
// fake motion on.
//
// Why displace geometry at all: the animated atlas tile is one 16x16 texture
// repeated on every water block, so it can never show a wave bigger than a
// block, every block crests at the same instant, and the surface stays
// geometrically flat. Moving the vertices as a function of *world* position
// fixes all three — a swell rolls across the whole lake, and the surface has
// real relief that catches the light. The tile still animates underneath and
// now reads as the fine chop on top of the swell.
//
// Watertightness: vertices are duplicated per face (each quad pushes its own
// four), so a shared corner exists several times over. The displacement is a
// pure function of world XZ, so every copy of a corner lands in the same place
// and no cracks open. `push_water_face` in mesh.rs flags which vertices are on
// the free surface via vertex-colour alpha; the rest stay pinned.

#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    forward_io::{Vertex, VertexOutput},
    mesh_view_bindings::globals,
}

// x = amplitude scale, y = speed scale, z = sink (how far the surface sits
// below the top of its block), w = choppiness (Gerstner horizontal pinch).
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> wave_params: vec4<f32>;

const TAU: f32 = 6.2831855;
// Gravity, for the deep-water dispersion relation w = sqrt(g*k). Tying speed to
// wavelength is what makes the long swell outrun the short chop instead of the
// whole field sliding along as one image.
const G: f32 = 9.8;

// Four waves: direction (unit), wavelength in blocks, amplitude in blocks.
// Wavelengths are mutually non-harmonic so the field doesn't visibly repeat.
const DIRS = array<vec2<f32>, 4>(
    vec2<f32>(1.0, 0.0),
    vec2<f32>(-0.6, 0.8),
    vec2<f32>(0.35, 0.94),
    vec2<f32>(-0.9, -0.44),
);
const LENS = array<f32, 4>(9.0, 5.3, 3.1, 1.9);
const AMPS = array<f32, 4>(0.055, 0.032, 0.019, 0.011);

/// Total Gerstner displacement at a world XZ position.
fn gerstner(p: vec2<f32>, t: f32, amp_scale: f32, chop: f32) -> vec3<f32> {
    var disp = vec3<f32>(0.0, 0.0, 0.0);
    let dirs = DIRS;
    let lens = LENS;
    let amps = AMPS;
    for (var i = 0; i < 4; i = i + 1) {
        let d = normalize(dirs[i]);
        let k = TAU / lens[i];
        let a = amps[i] * amp_scale;
        let phase = k * dot(d, p) + t * sqrt(G * k);
        // Vertical crest, plus the horizontal pinch that gives Gerstner waves
        // their sharp peaks and flat troughs (a plain sine looks like a rolling
        // blanket; this looks like water).
        disp.y = disp.y + a * sin(phase);
        let h = chop * a * cos(phase);
        disp.x = disp.x + d.x * h;
        disp.z = disp.z + d.y * h;
    }
    return disp;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    var world_position = mesh_functions::mesh_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0),
    );

    let amp_scale = wave_params.x;
    let speed = wave_params.y;
    let sink = wave_params.z;
    let chop = wave_params.w;

    // Alpha is the free-surface flag baked by the mesher: 1 = displace, 0 = pin.
    let surf = vertex.color.a;
    let t = globals.time * speed;
    let p = world_position.xz;
    let disp = gerstner(p, t, amp_scale, chop);

    world_position = vec4<f32>(
        world_position.xyz + (disp - vec3<f32>(0.0, sink, 0.0)) * surf,
        world_position.w,
    );

    var world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index,
    );

    // Recompute the normal for the up-facing surface. Without this the waves are
    // invisible: the geometry would move but every top quad would keep its flat
    // +Y normal and shade exactly as it did before. Two nearby samples give the
    // real tangent plane, choppiness included.
    if surf > 0.5 && vertex.normal.y > 0.5 {
        let e = 0.4;
        let dx = gerstner(p + vec2<f32>(e, 0.0), t, amp_scale, chop) - disp;
        let dz = gerstner(p + vec2<f32>(0.0, e), t, amp_scale, chop) - disp;
        let tangent_x = vec3<f32>(e, 0.0, 0.0) + dx;
        let tangent_z = vec3<f32>(0.0, 0.0, e) + dz;
        world_normal = normalize(cross(tangent_z, tangent_x));
    }

    out.world_position = world_position;
    out.world_normal = world_normal;
    out.position = position_world_to_clip(world_position.xyz);
    out.uv = vertex.uv;
    // Drop the surface flag out of alpha before the fragment stage — the PBR
    // path multiplies base colour by vertex colour, and a 0 here would erase
    // every pinned vertex.
    out.color = vec4<f32>(vertex.color.rgb, 1.0);
    out.instance_index = vertex.instance_index;
    return out;
}
