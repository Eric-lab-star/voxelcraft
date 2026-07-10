//! Little cube particles that burst out when a block is broken, then fall,
//! shrink, and despawn.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::block::Block;

const LIFETIME: f32 = 0.7;
const GRAVITY: f32 = 22.0;
const BASE_SIZE: f32 = 0.14;

/// Shared mesh + per-block materials so spawning particles allocates nothing.
#[derive(Resource)]
pub struct ParticleAssets {
    mesh: Handle<Mesh>,
    mats: HashMap<Block, Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct Particle {
    velocity: Vec3,
    life: f32,
}

pub fn setup_particle_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mut mats = HashMap::new();
    for block in [
        Block::Grass,
        Block::Dirt,
        Block::Stone,
        Block::Sand,
        Block::Wood,
        Block::Leaves,
        Block::Water,
    ] {
        mats.insert(
            block,
            materials.add(StandardMaterial {
                base_color: block.particle_color(),
                perceptual_roughness: 1.0,
                ..default()
            }),
        );
    }
    commands.insert_resource(ParticleAssets { mesh, mats });
}

/// Emit a burst of particles for a broken `block` centred at `pos`.
pub fn spawn_break_particles(
    commands: &mut Commands,
    assets: &ParticleAssets,
    block: Block,
    pos: Vec3,
    rng: &mut u32,
) {
    let Some(material) = assets.mats.get(&block) else {
        return;
    };
    for _ in 0..10 {
        let velocity = Vec3::new(
            (rand(rng) * 2.0 - 1.0) * 3.0,
            rand(rng) * 3.0 + 2.0,
            (rand(rng) * 2.0 - 1.0) * 3.0,
        );
        let offset = Vec3::new(rand(rng) - 0.5, rand(rng) - 0.5, rand(rng) - 0.5) * 0.6;
        commands.spawn((
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos + offset).with_scale(Vec3::splat(BASE_SIZE)),
            Particle {
                velocity,
                life: LIFETIME,
            },
        ));
    }
}

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut particle) in &mut query {
        particle.life -= dt;
        if particle.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        particle.velocity.y -= GRAVITY * dt;
        transform.translation += particle.velocity * dt;
        // Shrink as they age.
        let s = (particle.life / LIFETIME).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(BASE_SIZE * (0.3 + 0.7 * s));
    }
}

/// Cheap xorshift RNG -> [0,1). Seed is kept in a `Local` by the caller.
fn rand(state: &mut u32) -> f32 {
    if *state == 0 {
        *state = 0x9E37_79B9;
    }
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    (x as f32) / (u32::MAX as f32)
}
