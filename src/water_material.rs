//! The water material: `StandardMaterial` extended with a vertex shader that
//! rolls the surface with Gerstner waves (see `water.wgsl`).
//!
//! Only the vertex stage is replaced; lighting, transparency, shadows and fog
//! all stay on the standard PBR path, and the atlas UVs the mesher bakes are
//! passed through untouched.

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::{Shader, ShaderRef};

/// Fixed handle for the embedded water vertex shader.
const WATER_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3e7c9a55-1d4b-4f82-9a0e-6b2c8d41f907");

/// The concrete water material used by the chunk renderer.
pub type WaterMaterial = ExtendedMaterial<StandardMaterial, WaterExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct WaterExtension {
    /// x = amplitude scale, y = speed scale, z = sink, w = choppiness.
    #[uniform(100)]
    pub wave_params: Vec4,
}

impl WaterExtension {
    /// Amplitude sums to about 0.12 blocks across the four waves, and `SINK`
    /// drops the surface a little further than that. Together they keep the
    /// crest strictly below the top of its block: water that rose past it would
    /// poke through whatever sits above and split open at the shoreline.
    /// Sinking it also matches how water reads in Minecraft — a hair below the
    /// full block, so the bank stands slightly proud of it.
    pub const SINK: f32 = 0.13;

    pub fn new() -> Self {
        Self {
            wave_params: Vec4::new(1.0, 1.0, Self::SINK, 0.6),
        }
    }
}

impl Default for WaterExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialExtension for WaterExtension {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(WATER_SHADER_HANDLE)
    }
}

/// Embeds the water shader in the binary and registers the material.
pub struct WaterMaterialPlugin;

impl Plugin for WaterMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, WATER_SHADER_HANDLE, "water.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default());
    }
}
