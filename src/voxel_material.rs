//! The terrain material: `StandardMaterial` extended with a fragment shader
//! that tiles the atlas across greedy-meshed quads (see `voxel.wgsl`).
//!
//! The base material keeps all the usual PBR behaviour (lighting, shadows,
//! distance fog). The extension only adds one small uniform — the atlas
//! dimensions — and swaps in our fragment shader, which reads the per-block
//! *repeat* UVs and packed tile index the mesher produces.

use crate::texture::{COLS, ROWS, TILE};
use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::{Shader, ShaderRef};

/// Fixed handle for the embedded terrain fragment shader.
const VOXEL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("b2f4a1e0-9c3d-4a7b-8e21-5d6c7f0a1b23");

/// The concrete terrain material type used throughout the chunk renderer.
pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, VoxelExtension>;

/// Extension data for the terrain material: just the atlas layout the shader
/// needs to map a per-tile UV into the packed atlas image.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct VoxelExtension {
    /// x = columns, y = rows, z = half-texel inset, w = unused.
    #[uniform(100)]
    pub atlas_dims: Vec4,
}

impl VoxelExtension {
    pub fn new() -> Self {
        Self {
            atlas_dims: Vec4::new(
                COLS as f32,
                ROWS as f32,
                0.5 / TILE as f32,
                0.0,
            ),
        }
    }
}

impl Default for VoxelExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialExtension for VoxelExtension {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_SHADER_HANDLE)
    }
}

/// Embeds the terrain shader in the binary and registers the material.
pub struct VoxelMaterialPlugin;

impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, VOXEL_SHADER_HANDLE, "voxel.wgsl", Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
    }
}
