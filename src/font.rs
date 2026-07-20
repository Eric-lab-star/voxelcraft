//! The UI font.
//!
//! Bevy's built-in default is a Latin subset of Fira Mono with no Hangul at all,
//! so every Korean string in the menus renders as blank space. We ship Galmuri11
//! instead: a Hangul *pixel* font, whose bitmap-style letterforms sit far better
//! next to hand-painted voxel textures than a smooth UI sans would.
//!
//! The font is compiled into the binary rather than loaded from disk, which
//! keeps the promise the rest of the project makes — nothing to install, nothing
//! to ship alongside the .exe.

use bevy::asset::{AssetId, Assets};
use bevy::prelude::*;
use bevy::text::Font;

/// Galmuri11 by Lee Minseo, under the SIL Open Font License 1.1. The licence
/// travels with the font in `assets/fonts/Galmuri-OFL.txt`, and the Windows
/// bundle copies it next to the executable.
const GALMURI: &[u8] = include_bytes!("../assets/fonts/Galmuri11.ttf");

/// Galmuri is drawn on an 11-pixel grid. Text sized at a whole multiple of this
/// lands on that grid and stays crisp; anything in between gets interpolated and
/// the pixel edges turn to mush. Every font size in the UI is `PIXEL_GRID * n`.
pub const PIXEL_GRID: f32 = 11.0;

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        // Overwrite the asset behind the *default* font handle rather than
        // naming the font at each text node. Every `TextFont` in the project
        // leaves `font` at its default, so this fixes all of them at once and
        // can't be forgotten when a new label is added later.
        //
        // It has to happen here, while the app is still being built: `bevy_text`
        // registers a font into Parley's collection the first time it sees that
        // asset id and skips it forever after, so a swap made from a startup
        // system would load but never actually be used.
        let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
        fonts
            .insert(AssetId::default(), Font::from_bytes(GALMURI.to_vec()))
            .expect("replacing the default font asset");
    }
}
