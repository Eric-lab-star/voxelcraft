//! First-person hand viewmodel + a switchable camera perspective.
//!
//! In first person we show the player's arm and the block they're holding.
//! Following Bevy's `first_person_view_model` example, the hand lives on its own
//! render layer drawn by a second *view-model* camera that is a child of the
//! main camera. That camera renders after the world with its own depth, so the
//! hand is always drawn on top and never clips into nearby terrain.
//!
//! Press **F5** to cycle first-person → third-person (behind) → third-person
//! (front); the third-person modes pull the camera off the eye and reveal a
//! simple body avatar standing where the player is.

use bevy::camera::visibility::RenderLayers;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::block::Block;
use crate::hotbar::Hotbar;
use crate::player::Player;
use crate::texture::{atlas_uv, block_tile, BlockAtlas};
use crate::world::World;

/// How far the camera sits from the eye in the third-person modes.
const THIRD_DISTANCE: f32 = 4.5;

/// Render layer the first-person hand lives on (the world is on layer 0).
const HAND_LAYER: usize = 1;

/// Which camera perspective is active. F5 cycles through these in order.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    #[default]
    FirstPerson,
    ThirdBack,
    ThirdFront,
}

impl ViewMode {
    fn next(self) -> Self {
        match self {
            ViewMode::FirstPerson => ViewMode::ThirdBack,
            ViewMode::ThirdBack => ViewMode::ThirdFront,
            ViewMode::ThirdFront => ViewMode::FirstPerson,
        }
    }

    fn is_first_person(self) -> bool {
        self == ViewMode::FirstPerson
    }
}

/// A first-person viewmodel piece (arm or held block). A child of the main
/// camera; `base` is its resting local transform, animated during a swing.
#[derive(Component)]
pub struct ViewHand {
    base: Transform,
}

/// How long one hand swing lasts, in seconds.
const SWING_TIME: f32 = 0.35;

/// Counts down while the hand is mid-swing (0 = idle, hand hidden).
#[derive(Resource, Default)]
pub struct SwingState {
    timer: f32,
}

/// Marks the held-block cube so its mesh can be rebuilt on hotbar changes.
#[derive(Component)]
pub struct HeldItem;

/// Root of the third-person body avatar (a world entity that follows the
/// player).
#[derive(Component)]
pub struct PlayerBody;

// --- Setup -----------------------------------------------------------------

pub fn setup_viewmodel(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atlas: Res<BlockAtlas>,
    hotbar: Res<Hotbar>,
    player: Query<&Player>,
    camera: Query<Entity, With<Player>>,
) {
    // The held block: the atlas texture, shaded per-face by baked vertex colours.
    let held_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas.image.clone()),
        ..default()
    });

    let held_mesh = meshes.add(block_cube_mesh(hotbar.block()));

    // Local offset in view space (camera looks down -Z, +X right, +Y up), so the
    // held block rides at the bottom-right of the screen. As a child of the main
    // camera it tracks the eye automatically.
    let held_local = Transform::from_translation(Vec3::new(0.45, -0.28, -1.0))
        .with_scale(Vec3::splat(0.44))
        .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.6, -0.2, 0.15));

    let layer = RenderLayers::layer(HAND_LAYER);
    if let Ok(cam) = camera.single() {
        commands.entity(cam).with_children(|c| {
            // The held block. Starts hidden; shown only mid-swing.
            c.spawn((
                HeldItem,
                ViewHand { base: held_local },
                Mesh3d(held_mesh),
                MeshMaterial3d(held_mat),
                held_local,
                Visibility::Hidden,
                layer,
                NotShadowCaster,
            ));
        });
    }

    // The third-person body avatar. Positioned/oriented each frame by
    // `apply_view_mode`; hidden until a third-person mode is active.
    let skin = materials.add(StandardMaterial {
        base_color: Color::srgb(0.80, 0.62, 0.47),
        ..default()
    });
    let shirt = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.50, 0.85),
        ..default()
    });
    let pants = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.28, 0.50),
        ..default()
    });
    let head = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    let torso = meshes.add(Cuboid::new(0.55, 0.75, 0.3));
    let limb = meshes.add(Cuboid::new(0.2, 0.75, 0.25));

    let start = player.single().map(|p| p.center).unwrap_or(Vec3::ZERO);
    commands
        .spawn((
            PlayerBody,
            Transform::from_translation(start),
            Visibility::Hidden,
        ))
        .with_children(|b| {
            // Offsets are relative to the collision-box centre.
            b.spawn((
                Mesh3d(head.clone()),
                MeshMaterial3d(skin.clone()),
                Transform::from_xyz(0.0, 0.85, 0.0),
            ));
            b.spawn((
                Mesh3d(torso),
                MeshMaterial3d(shirt),
                Transform::from_xyz(0.0, 0.15, 0.0),
            ));
            b.spawn((
                Mesh3d(limb.clone()),
                MeshMaterial3d(skin.clone()),
                Transform::from_xyz(0.38, 0.15, 0.0),
            ));
            b.spawn((
                Mesh3d(limb.clone()),
                MeshMaterial3d(skin),
                Transform::from_xyz(-0.38, 0.15, 0.0),
            ));
            b.spawn((
                Mesh3d(limb.clone()),
                MeshMaterial3d(pants.clone()),
                Transform::from_xyz(0.15, -0.6, 0.0),
            ));
            b.spawn((
                Mesh3d(limb),
                MeshMaterial3d(pants),
                Transform::from_xyz(-0.15, -0.6, 0.0),
            ));
        });
}

// --- Per-frame systems -----------------------------------------------------

/// F5 cycles the camera perspective.
pub fn cycle_view_mode(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<ViewMode>) {
    if keys.just_pressed(KeyCode::F5) {
        *view = view.next();
    }
}

/// Position the camera for the active view mode and keep the body avatar under
/// the player. Runs after `player_physics`, which writes the eye transform.
pub fn apply_view_mode(
    view: Res<ViewMode>,
    world: Res<World>,
    mut camera: Query<(&Player, &mut Transform), Without<PlayerBody>>,
    mut body: Query<&mut Transform, (With<PlayerBody>, Without<Player>)>,
) {
    let Ok((player, mut cam)) = camera.single_mut() else {
        return;
    };
    let eye = player.eye();
    let rot = player.look_rotation();
    let forward = player.forward();

    match *view {
        ViewMode::FirstPerson => {
            cam.translation = eye;
            cam.rotation = rot;
        }
        ViewMode::ThirdBack => {
            let dist = free_distance(&world, eye, -forward, THIRD_DISTANCE);
            cam.translation = eye - forward * dist;
            cam.rotation = rot;
        }
        ViewMode::ThirdFront => {
            let dist = free_distance(&world, eye, forward, THIRD_DISTANCE);
            cam.translation = eye + forward * dist;
            // Look back toward the player.
            cam.look_at(eye, Vec3::Y);
        }
    }

    if let Ok(mut body) = body.single_mut() {
        body.translation = player.center;
        body.rotation = Quat::from_rotation_y(player.yaw);
    }
}

/// Show the third-person body avatar only in the third-person modes. The
/// first-person hand's visibility is driven by `swing_hand` instead.
pub fn update_view_visibility(
    view: Res<ViewMode>,
    mut body: Query<&mut Visibility, With<PlayerBody>>,
) {
    if !view.is_changed() {
        return;
    }
    let first = view.is_first_person();
    for mut v in &mut body {
        *v = if first { Visibility::Hidden } else { Visibility::Visible };
    }
}

/// A left- or right-click starts a hand swing (independent of whether the
/// raycast hit anything — you always swing at the air too).
pub fn swing_input(buttons: Res<ButtonInput<MouseButton>>, mut swing: ResMut<SwingState>) {
    if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) {
        swing.timer = SWING_TIME;
    }
}

/// Animate the hand: hidden when idle, and while a swing is running it appears
/// and jabs forward-and-down in an arc (only in first person).
pub fn swing_hand(
    time: Res<Time>,
    view: Res<ViewMode>,
    mut swing: ResMut<SwingState>,
    mut hands: Query<(&ViewHand, &mut Transform, &mut Visibility)>,
) {
    swing.timer = (swing.timer - time.delta_secs()).max(0.0);
    let active = view.is_first_person() && swing.timer > 0.0;

    // Swing progress 0->1, shaped into a 0->1->0 arc so the hand reaches out and
    // pulls back within the swing.
    let progress = 1.0 - (swing.timer / SWING_TIME).clamp(0.0, 1.0);
    let arc = (progress * std::f32::consts::PI).sin();

    for (hand, mut transform, mut vis) in &mut hands {
        if active {
            *vis = Visibility::Inherited;
            *transform = Transform {
                translation: hand.base.translation
                    + Vec3::new(-0.12 * arc, -0.18 * arc, -0.3 * arc),
                rotation: hand.base.rotation * Quat::from_rotation_x(-1.5 * arc),
                scale: hand.base.scale,
            };
        } else if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
    }
}

/// Rebuild the held-block cube when the hotbar selection changes.
pub fn update_held_item(
    hotbar: Res<Hotbar>,
    held: Query<&Mesh3d, With<HeldItem>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !hotbar.is_changed() {
        return;
    }
    let Ok(mesh) = held.single() else {
        return;
    };
    let _ = meshes.insert(&mesh.0, block_cube_mesh(hotbar.block()));
}

// --- Helpers ---------------------------------------------------------------

/// Distance the camera can move from `origin` along `dir` before hitting a
/// solid block, capped at `max`. Keeps the third-person camera out of terrain.
fn free_distance(world: &World, origin: Vec3, dir: Vec3, max: f32) -> f32 {
    let mut d = 0.2;
    while d < max {
        let p = origin + dir * d;
        if world
            .get(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32)
            .blocks_movement()
        {
            return (d - 0.1).max(0.0);
        }
        d += 0.1;
    }
    max
}

// Unit-cube face geometry (0..1 corners), matching the terrain mesher so tiles
// sit upright. Order: -X, +X, -Y, +Y, -Z, +Z.
const FACE_NORMALS: [[f32; 3]; 6] = [
    [-1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, 1.0],
];

const FACE_CORNERS: [[[f32; 3]; 4]; 6] = [
    [[0., 0., 1.], [0., 0., 0.], [0., 1., 0.], [0., 1., 1.]],
    [[1., 0., 0.], [1., 0., 1.], [1., 1., 1.], [1., 1., 0.]],
    [[0., 0., 1.], [1., 0., 1.], [1., 0., 0.], [0., 0., 0.]],
    [[0., 1., 0.], [1., 1., 0.], [1., 1., 1.], [0., 1., 1.]],
    [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
    [[1., 0., 1.], [0., 0., 1.], [0., 1., 1.], [1., 1., 1.]],
];

/// Same directional shading the terrain uses, so the held block matches it.
const FACE_SHADE: [f32; 6] = [0.65, 0.65, 0.5, 1.0, 0.8, 0.8];

/// Build a centred unit cube textured with `block`'s atlas tiles, with per-face
/// shading baked into vertex colours.
fn block_cube_mesh(block: Block) -> Mesh {
    build_cube(Vec3::ONE, |f| Some(block_tile(block, f)), [1.0; 3])
}

fn build_cube(size: Vec3, tile_of: impl Fn(usize) -> Option<u32>, tint: [f32; 3]) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::mesh::{Indices, PrimitiveTopology};

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut colors = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for f in 0..6 {
        // Keep textures upright: side faces list bottom corners then top; the
        // top/bottom faces lie flat.
        let corner_uv = if f == 2 || f == 3 {
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
        } else {
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]
        };
        let tile = tile_of(f);
        let s = FACE_SHADE[f];
        let start = positions.len() as u32;
        for (ci, corner) in FACE_CORNERS[f].iter().enumerate() {
            positions.push([
                (corner[0] - 0.5) * size.x,
                (corner[1] - 0.5) * size.y,
                (corner[2] - 0.5) * size.z,
            ]);
            normals.push(FACE_NORMALS[f]);
            uvs.push(match tile {
                Some(t) => atlas_uv(t, corner_uv[ci][0], corner_uv[ci][1]),
                None => [0.0, 0.0],
            });
            colors.push([s * tint[0], s * tint[1], s * tint[2], 1.0]);
        }
        indices.extend_from_slice(&[start, start + 1, start + 2, start, start + 2, start + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
