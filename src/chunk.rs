//! Spawns the render entities for each chunk column (one opaque + one water
//! mesh) and rebuilds any chunk marked dirty after an edit.

use std::collections::{HashMap, HashSet};

use crate::mesh::{build_chunk_meshes, ChunkMeshes};
use crate::player::Player;
use crate::texture::BlockAtlas;
use crate::voxel_material::{TerrainMaterial, VoxelExtension};
use crate::water_material::{WaterExtension, WaterMaterial};
use crate::world::{MapKind, World, CHUNK_SIZE, WORLD_X, WORLD_Z};
use bevy::pbr::Material;
use bevy::prelude::*;

/// The two render entities for a chunk (either may be absent).
#[derive(Default, Clone, Copy)]
pub struct ChunkPair {
    pub opaque: Option<Entity>,
    pub water: Option<Entity>,
    pub plants: Option<Entity>,
}

/// Maps chunk coordinates -> its render entities.
#[derive(Resource, Default)]
pub struct ChunkEntities(pub HashMap<(i32, i32), ChunkPair>);

/// Chunks whose meshes need rebuilding this frame.
#[derive(Resource, Default)]
pub struct DirtyChunks(pub HashSet<(i32, i32)>);

/// The opaque terrain material (greedy-meshed, atlas-tiled) and the translucent
/// water material.
#[derive(Resource)]
pub struct ChunkMaterials {
    pub terrain: Handle<TerrainMaterial>,
    pub water: Handle<WaterMaterial>,
    pub plants: Handle<StandardMaterial>,
}

/// Convert a world block X/Z to its chunk coordinate.
pub fn chunk_of(x: i32, z: i32) -> (i32, i32) {
    (x.div_euclid(CHUNK_SIZE), z.div_euclid(CHUNK_SIZE))
}

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut plant_materials: ResMut<Assets<StandardMaterial>>,
    atlas: Res<BlockAtlas>,
) {
    // The title screen is drawn over a real world, so we always need one built
    // by the time it appears. Generate the meadow: it is the cheapest map and
    // makes a fine backdrop, and whichever map the player picks replaces it.
    let world = World::generate(MapKind::Meadow, 1337);

    let terrain = terrain_materials.add(TerrainMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(atlas.image.clone()),
            perceptual_roughness: 0.9,
            // Voxel faces are single-sided in our mesh; render both sides so we
            // never have to worry about winding order.
            cull_mode: None,
            double_sided: true,
            ..default()
        },
        extension: VoxelExtension::new(),
    });
    let water = water_materials.add(WaterMaterial {
        base: StandardMaterial {
            // Neutral tint so the water tile's own colour comes through
            // unchanged; the texture, not the material, decides the hue. Alpha
            // lets the lakebed read through — but only so far: a bright sandy
            // bed at much below this washes the blue out to grey.
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.9),
            base_color_texture: Some(atlas.image.clone()),
            alpha_mode: AlphaMode::Blend,
            // Damp, not mirrored. At 0.08 roughness with the default 0.5
            // reflectance the sun laid a broad white sheen right across the
            // up-facing surface, and seen from above that sheen *was* the water
            // — the blue underneath never got through. Real water reflects about
            // 0.02 head-on; 0.12 keeps a highlight without bleaching the hue.
            perceptual_roughness: 0.34,
            reflectance: 0.12,
            cull_mode: None,
            double_sided: true,
            ..default()
        },
        extension: WaterExtension::new(),
    });
    // Plants: plain PBR, no custom shader — the mesh bakes atlas UVs directly, so
    // there is nothing to tile. `Mask` is the whole point: it discards the
    // transparent pixels outright rather than blending them, which keeps the
    // cutout crisp and, unlike `Blend`, needs no back-to-front sorting — crossed
    // quads inside one mesh could never be sorted correctly anyway.
    let plants = plant_materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas.image.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        perceptual_roughness: 0.95,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let mats = ChunkMaterials {
        terrain,
        water,
        plants,
    };

    let chunks_x = WORLD_X / CHUNK_SIZE;
    let chunks_z = WORLD_Z / CHUNK_SIZE;
    let mut entities = HashMap::new();

    for cz in 0..chunks_z {
        for cx in 0..chunks_x {
            let built = build_chunk_meshes(&world, cx, cz);
            let pair = spawn_pair(&mut commands, &mut meshes, &mats, built);
            if pair.opaque.is_some() || pair.water.is_some() || pair.plants.is_some() {
                entities.insert((cx, cz), pair);
            }
        }
    }

    // Spawn the player (camera) standing on grass above the waterline.
    let spawn = world.find_spawn();
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        // Match the view-model camera's MSAA (both `Off`). If the two cameras
        // that share this window disagree on MSAA, the view-model pass — which
        // loads instead of clears (`ClearColorConfig::None`) — reads a stale
        // buffer, so the hand smears/ghosts old frames instead of being redrawn.
        bevy::render::view::Msaa::Off,
        // A visibility root so the child first-person hand renders. The HUD/menu
        // are drawn by a separate 2D UI camera (see `setup_scene`) that sits on
        // top of everything, so this world camera is deliberately NOT the UI
        // camera.
        Visibility::default(),
        // Distance fog blends far terrain into the sky, so the world reads as
        // stretching to the horizon. Its colour tracks the sky (day/night).
        DistanceFog {
            color: Color::srgb(0.53, 0.74, 0.92),
            falloff: FogFalloff::Linear {
                start: 90.0,
                end: 230.0,
            },
            ..default()
        },
        Player::new(spawn),
        // The view-model camera: renders only the hand layer (1), after the
        // world, on top, with its own depth so the hand never clips terrain.
        // `Msaa::Off` is required — a second camera whose MSAA doesn't match the
        // window's writeback silently fails to composite.
        children![(
            Camera3d::default(),
            Camera {
                order: 1,
                // Load (don't clear) so the world drawn by the main camera
                // stays; we only draw the hand on top.
                clear_color: bevy::camera::ClearColorConfig::None,
                ..default()
            },
            Projection::from(PerspectiveProjection {
                fov: 70.0_f32.to_radians(),
                ..default()
            }),
            bevy::render::view::Msaa::Off,
            bevy::camera::visibility::RenderLayers::layer(1),
        )],
    ));

    commands.insert_resource(world);
    commands.insert_resource(ChunkEntities(entities));
    commands.insert_resource(DirtyChunks::default());
    commands.insert_resource(mats);
}

/// Spawn fresh entities for a chunk's meshes.
fn spawn_pair(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &ChunkMaterials,
    built: ChunkMeshes,
) -> ChunkPair {
    let opaque = built.opaque.map(|mesh| {
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(mats.terrain.clone()),
                Transform::IDENTITY,
            ))
            .id()
    });
    let water = built.water.map(|mesh| {
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(mats.water.clone()),
                Transform::IDENTITY,
            ))
            .id()
    });
    let plants = built.plants.map(|mesh| {
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(mats.plants.clone()),
                Transform::IDENTITY,
            ))
            .id()
    });
    ChunkPair {
        opaque,
        water,
        plants,
    }
}

pub fn rebuild_dirty_chunks(
    world: Res<World>,
    mats: Res<ChunkMaterials>,
    mut dirty: ResMut<DirtyChunks>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_query: Query<&Mesh3d>,
    mut commands: Commands,
) {
    if dirty.0.is_empty() {
        return;
    }

    for &coord in dirty.0.iter() {
        let built = build_chunk_meshes(&world, coord.0, coord.1);
        let mut pair = chunk_entities.0.get(&coord).copied().unwrap_or_default();

        update_slot(
            &mut commands,
            &mut meshes,
            &mesh_query,
            &mats.terrain,
            &mut pair.opaque,
            built.opaque,
        );
        update_slot(
            &mut commands,
            &mut meshes,
            &mesh_query,
            &mats.water,
            &mut pair.water,
            built.water,
        );
        update_slot(
            &mut commands,
            &mut meshes,
            &mesh_query,
            &mats.plants,
            &mut pair.plants,
            built.plants,
        );

        if pair.opaque.is_some() || pair.water.is_some() || pair.plants.is_some() {
            chunk_entities.0.insert(coord, pair);
        } else {
            chunk_entities.0.remove(&coord);
        }
    }

    dirty.0.clear();
}

/// Reconcile one entity slot with a freshly-built (or now-empty) mesh.
fn update_slot<M: Material>(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mesh_query: &Query<&Mesh3d>,
    material: &Handle<M>,
    slot: &mut Option<Entity>,
    new_mesh: Option<Mesh>,
) {
    match (*slot, new_mesh) {
        (Some(entity), Some(mesh)) => {
            if let Ok(handle) = mesh_query.get(entity) {
                let _ = meshes.insert(&handle.0, mesh);
            }
        }
        (Some(entity), None) => {
            commands.entity(entity).despawn();
            *slot = None;
        }
        (None, Some(mesh)) => {
            let entity = commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                ))
                .id();
            *slot = Some(entity);
        }
        (None, None) => {}
    }
}
