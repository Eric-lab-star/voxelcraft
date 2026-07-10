//! A drifting flat cloud layer high above the world.

use bevy::asset::RenderAssetUsages;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

use crate::texture::build_clouds;
use crate::world::{WORLD_X, WORLD_Z};

/// Height of the cloud layer, above the terrain but low enough to see when
/// glancing up or toward the horizon.
const CLOUD_Y: f32 = 78.0;
/// Half-size of the cloud plane (much larger than the view distance).
const HALF: f32 = 420.0;
/// How many times the cloud texture repeats across the plane. Fewer repeats =
/// larger clouds and a much less obvious grid pattern.
const TILES: f32 = 7.0;

#[derive(Component)]
pub struct Clouds;

pub fn setup_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = images.add(build_clouds());
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(cloud_plane())),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        bevy::camera::visibility::NoFrustumCulling,
        Clouds,
    ));
}

/// A single large horizontal quad centred over the world.
fn cloud_plane() -> Mesh {
    let cx = WORLD_X as f32 / 2.0;
    let cz = WORLD_Z as f32 / 2.0;
    let positions = vec![
        [cx - HALF, CLOUD_Y, cz - HALF],
        [cx + HALF, CLOUD_Y, cz - HALF],
        [cx + HALF, CLOUD_Y, cz + HALF],
        [cx - HALF, CLOUD_Y, cz + HALF],
    ];
    let normals = vec![[0.0, -1.0, 0.0]; 4];
    let uvs = vec![[0.0, 0.0], [TILES, 0.0], [TILES, TILES], [0.0, TILES]];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

/// Slowly scroll the cloud texture so the clouds appear to drift.
pub fn drift_clouds(
    time: Res<Time>,
    clouds: Query<&MeshMaterial3d<StandardMaterial>, With<Clouds>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let t = time.elapsed_secs();
    for handle in &clouds {
        if let Some(mut material) = materials.get_mut(&handle.0) {
            // Noticeably drift across the sky (with a slight sideways component).
            material.uv_transform = Affine2::from_translation(Vec2::new(t * 0.022, t * 0.009));
        }
    }
}
