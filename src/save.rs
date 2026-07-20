//! World persistence with numbered save slots. F5/F9 use slot 1; the pause menu
//! can target any slot.

use bevy::prelude::*;

use crate::chunk::DirtyChunks;
use crate::menu::Toast;
use crate::world::{MapKind, World, CHUNK_SIZE, WORLD_X, WORLD_Z};

/// How many save slots exist.
pub const NUM_SLOTS: usize = 3;

/// File name for a given slot (1-based).
pub fn slot_path(slot: usize) -> String {
    format!("world_{slot}.sav")
}

/// Whether a slot has a saved world on disk.
pub fn slot_exists(slot: usize) -> bool {
    std::path::Path::new(&slot_path(slot)).exists()
}

/// Save the world into `slot`.
pub fn save_world(world: &World, slot: usize) -> bool {
    match world.save(&slot_path(slot)) {
        Ok(()) => {
            info!("world saved to slot {slot}");
            true
        }
        Err(e) => {
            warn!("failed to save world: {e}");
            false
        }
    }
}

/// Load `slot` into the world, queuing every chunk for a rebuild. Returns
/// whether a valid save was loaded.
pub fn load_world(world: &mut World, dirty: &mut DirtyChunks, slot: usize) -> bool {
    match World::load(&slot_path(slot)) {
        Some(mut loaded) => {
            // A save written before plants existed has none anywhere; scatter
            // them once so an old world still gets its meadows. `decorate` only
            // fills air above grass, so nothing already built is disturbed, and
            // after the next save the world has plants and this never fires for
            // it again — so plants the player pulled up stay gone.
            if !loaded.has_plants() {
                loaded.decorate(1337);
            }
            *world = loaded;
            mark_all_dirty(dirty);
            info!("world loaded from slot {slot}");
            true
        }
        None => false,
    }
}

/// Replace the world with a freshly generated map of `kind`.
pub fn new_world(world: &mut World, dirty: &mut DirtyChunks, kind: MapKind, seed: u32) {
    *world = World::generate(kind, seed);
    mark_all_dirty(dirty);
    info!("generated a new {:?} world", kind);
}

/// Queue every chunk for a rebuild — needed whenever the world is wholesale
/// replaced rather than edited a block at a time.
fn mark_all_dirty(dirty: &mut DirtyChunks) {
    for cz in 0..(WORLD_Z / CHUNK_SIZE) {
        for cx in 0..(WORLD_X / CHUNK_SIZE) {
            dirty.0.insert((cx, cz));
        }
    }
}

pub fn save_load_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<World>,
    mut dirty: ResMut<DirtyChunks>,
    mut toast: ResMut<Toast>,
) {
    if keys.just_pressed(KeyCode::F5) {
        if save_world(&world, 1) {
            toast.show("Saved to slot 1");
        }
    }
    if keys.just_pressed(KeyCode::F9) {
        if load_world(&mut world, &mut dirty, 1) {
            toast.show("Loaded slot 1");
        } else {
            toast.show("Slot 1 is empty");
        }
    }
}
