//! voxelcraft — a minimal Minecraft-like voxel sandbox built on Bevy.
//!
//! Controls:
//!   Mouse        look
//!   WASD         walk   (Ctrl = sprint)
//!   Space        jump; in water, swim up (against a wall, climb out)
//!   Shift        dive
//!   Space Space  toggle flight (developer mode) — then Space up, Shift down
//!   Left click   break block
//!   Right click  place the selected block
//!   1-0 / wheel  pick a hotbar slot
//!   Tab          pause menu (save / load / quit)
//!   Escape       release / recapture the mouse cursor

mod block;
mod chunk;
mod clouds;
mod daynight;
mod font;
mod hotbar;
mod interaction;
mod joseon;
mod menu;
mod mesh;
mod particles;
mod player;
mod save;
mod texture;
mod viewmodel;
mod voxel_material;
mod water;
mod water_material;
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
        // Swaps in a Hangul-capable UI font. Must come after `DefaultPlugins`,
        // which is what creates the font assets it replaces.
        .add_plugins(font::FontPlugin)
        // Registers the greedy-mesh terrain material + its embedded shader.
        .add_plugins(voxel_material::VoxelMaterialPlugin)
        // Registers the Gerstner-wave water material + its embedded shader.
        .add_plugins(water_material::WaterMaterialPlugin)
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
        .init_resource::<viewmodel::ViewMode>()
        .init_resource::<viewmodel::SwingState>()
        .init_resource::<viewmodel::HeldRotDebug>()
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
                interaction::setup_sign_label,
                interaction::setup_sign_names,
                setup_scene,
                player::grab_cursor,
            ),
        )
        // Spawn the hand viewmodel after the camera entity exists.
        .add_systems(PostStartup, viewmodel::setup_viewmodel)
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
                interaction::show_sign_name,
                save::save_load_input,
                particles::update_particles,
                water::simulate_water,
                water::animate_water,
                water::underwater_effect,
                water::update_bubbles,
                chunk::rebuild_dirty_chunks,
                viewmodel::update_held_item,
                viewmodel::update_view_visibility,
                viewmodel::update_viewmodel_visibility,
                viewmodel::swing_hand,
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
                viewmodel::swing_input,
                viewmodel::cycle_view_mode,
                viewmodel::debug_held_rotation,
                // Positions the camera per view mode; must run after physics
                // writes the eye transform.
                viewmodel::apply_view_mode.after(player::player_physics),
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
    // Sun — its transform/colour are driven by the day/night cycle. It lights
    // both the world (layer 0) and the first-person hand (layer 1).
    commands.spawn((
        DirectionalLight {
            illuminance: 11_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(60.0, 80.0, 35.0).looking_at(Vec3::ZERO, Vec3::Y),
        daynight::Sun,
        bevy::camera::visibility::RenderLayers::from_layers(&[0, 1]),
    ));

    // (The player/camera is spawned in `setup_world`, where terrain height is
    // known, so it can be placed standing on the ground.)

    // Dedicated UI camera, drawn last (order 2) so the HUD, hotbar and menu
    // always sit on top of the first-person hand (which the view-model camera
    // draws at order 1). A 2D camera never renders the 3D world, so it can't
    // paint terrain back over the hand; `ClearColorConfig::None` keeps the
    // world+hand underneath, and `Msaa::Off` matches the other two cameras so
    // this load-not-clear pass doesn't read a stale buffer.
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            clear_color: bevy::camera::ClearColorConfig::None,
            ..default()
        },
        bevy::render::view::Msaa::Off,
        bevy::ui::IsDefaultUiCamera,
    ));

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
