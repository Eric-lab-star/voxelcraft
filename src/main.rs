//! voxelcraft — a minimal Minecraft-like voxel sandbox built on Bevy.
//!
//! Controls:
//!   Mouse        look
//!   WASD         walk   (Ctrl = sprint)
//!   Space        jump
//!   Left click   break block
//!   Right click  place block (stone)
//!   Escape       release / recapture the mouse cursor

mod block;
mod chunk;
mod clouds;
mod daynight;
mod hotbar;
mod interaction;
mod menu;
mod mesh;
mod particles;
mod player;
mod save;
mod texture;
mod voxel_material;
mod water;
mod world;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WINIT_WINDOWS;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "voxelcraft".into(),
                        ..default()
                    }),
                    ..default()
                })
                // Nearest-neighbour sampling keeps the pixel-art textures crisp.
                .set(ImagePlugin::default_nearest()),
        )
        // Registers the greedy-mesh terrain material + its embedded shader.
        .add_plugins(voxel_material::VoxelMaterialPlugin)
        .insert_resource(ClearColor(Color::srgb(0.53, 0.74, 0.92)))
        // Lower ambient so the sun and the per-face shading create real
        // contrast between terrain layers instead of a flat, evenly-lit look.
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 170.0,
            ..default()
        })
        .init_resource::<daynight::GameClock>()
        .init_resource::<water::WaterSim>()
        .init_resource::<menu::MenuState>()
        .init_resource::<menu::Toast>()
        // Build the shared texture atlas before anything that references it.
        .add_systems(PreStartup, texture::setup_atlas)
        .add_systems(
            Startup,
            (
                chunk::setup_world,
                hotbar::setup_hotbar_ui,
                particles::setup_particle_assets,
                water::setup_water_fx,
                clouds::setup_clouds,
                menu::setup_menu,
                setup_scene,
                player::grab_cursor,
            ),
        )
        // Menu + always-on world systems.
        .add_systems(
            Update,
            (
                apply_window_icon,
                daynight::day_night,
                clouds::drift_clouds,
                menu::toggle_menu,
                menu::apply_menu_state,
                menu::menu_button_actions,
                menu::update_toast,
                save::save_load_input,
                particles::update_particles,
                water::simulate_water,
                water::underwater_effect,
                water::update_bubbles,
                chunk::rebuild_dirty_chunks,
            ),
        )
        // Player/input systems — paused while the menu is open.
        .add_systems(
            Update,
            (
                player::player_look,
                player::player_physics,
                player::toggle_cursor_grab,
                hotbar::select_slot,
                hotbar::update_selection,
                interaction::edit_blocks,
                interaction::highlight_target,
            )
                .run_if(menu::game_active),
        )
        .run();
}

/// Set the window/taskbar icon once the winit window exists. Runs as an
/// exclusive system (main thread) so it can touch the `WINIT_WINDOWS`
/// thread-local, and stops itself after succeeding.
fn apply_window_icon(world: &mut World, mut done: Local<bool>) {
    if *done {
        return;
    }
    let mut query = world.query_filtered::<Entity, With<PrimaryWindow>>();
    let Ok(entity) = query.single(world) else {
        return;
    };

    let (rgba, w, h) = texture::build_icon();
    let Ok(icon) = winit::window::Icon::from_rgba(rgba, w, h) else {
        *done = true; // bad icon data — don't keep retrying
        return;
    };

    WINIT_WINDOWS.with_borrow(|winit_windows| {
        if let Some(window) = winit_windows.get_window(entity) {
            window.set_window_icon(Some(icon));
            *done = true;
        }
    });
}

fn setup_scene(mut commands: Commands) {
    // Sun — its transform/colour are driven by the day/night cycle.
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(60.0, 80.0, 35.0).looking_at(Vec3::ZERO, Vec3::Y),
        daynight::Sun,
    ));

    // (The player/camera is spawned in `setup_world`, where terrain height is
    // known, so it can be placed standing on the ground.)

    // Crosshair: a small centred dot.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(5.0),
                    height: Val::Px(5.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            ));
        });
}
