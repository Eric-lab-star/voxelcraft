//! A Minecraft-style hotbar: a row of block slots at the bottom of the screen.
//! Pick the active block with the number keys or the mouse wheel; the selected
//! block is what right-click places.
//!
//! There are more blocks than fit across the screen, so the bar is a sliding
//! *window* onto the full list rather than the whole thing. It shows nine cells
//! and scrolls the window along to keep the selection in view — no paging keys
//! to learn, and the bar stays the same size however many blocks get added.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::block::Block;
use crate::texture::{block_tile, BlockAtlas};

/// Every placeable block, in bar order. The first entry is the empty hand
/// (`None`); selecting it means you hold nothing and see your bare arm.
pub const SLOTS: [Option<Block>; 21] = [
    None,
    Some(Block::Grass),
    Some(Block::Dirt),
    Some(Block::Stone),
    Some(Block::Sand),
    Some(Block::Wood),
    Some(Block::Leaves),
    Some(Block::Water),
    Some(Block::TallGrass),
    Some(Block::RedFlower),
    Some(Block::YellowFlower),
    Some(Block::RoofTile),
    Some(Block::RoofRidge),
    Some(Block::Thatch),
    Some(Block::Plaster),
    Some(Block::ClayWall),
    Some(Block::Paper),
    Some(Block::Dancheong),
    Some(Block::RedPillar),
    Some(Block::Granite),
    Some(Block::Road),
];

/// How many cells the bar shows at once.
pub const VISIBLE: usize = 9;

/// Digit keys, one per visible cell.
const DIGIT_KEYS: [KeyCode; VISIBLE] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// Which block is selected, as an index into [`SLOTS`].
#[derive(Resource)]
pub struct Hotbar {
    pub selected: usize,
}

impl Default for Hotbar {
    fn default() -> Self {
        // Start holding grass, not the empty hand.
        Self { selected: 1 }
    }
}

impl Hotbar {
    /// The block the player will place, or `None` when the hand is empty.
    pub fn block(&self) -> Option<Block> {
        SLOTS[self.selected]
    }

    /// Index of the leftmost visible slot. The window keeps the selection
    /// centred until it runs into either end of the list, then stops — so the
    /// bar doesn't jitter when you scroll near the edges.
    pub fn window_start(&self) -> usize {
        let last_start = SLOTS.len().saturating_sub(VISIBLE);
        self.selected.saturating_sub(VISIBLE / 2).min(last_start)
    }
}

/// A cell of the bar, by *visible* position — not by slot, since which slot a
/// cell shows changes as the window slides.
#[derive(Component)]
pub struct HotbarCell(usize);

/// The block icon inside a cell.
#[derive(Component)]
pub struct HotbarIcon(usize);

pub fn setup_hotbar_ui(mut commands: Commands, atlas: Res<BlockAtlas>) {
    commands.insert_resource(Hotbar::default());

    // Full-screen anchor: centre horizontally, pin to the bottom edge.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    padding: UiRect::all(Val::Px(4.0)),
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
            ))
            .with_children(|bar| {
                for cell in 0..VISIBLE {
                    bar.spawn((
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                        HotbarCell(cell),
                    ))
                    .with_children(|slot| {
                        // Every cell gets an icon, even the one that starts on
                        // the empty hand: the window slides, so any cell may
                        // need to show a block later. `update_selection` hides
                        // it when the slot it lands on is the empty hand.
                        slot.spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            ImageNode::from_atlas_image(
                                atlas.image.clone(),
                                TextureAtlas {
                                    layout: atlas.layout.clone(),
                                    index: 0,
                                },
                            ),
                            HotbarIcon(cell),
                        ));
                    });
                }
            });
        });
}

/// Number keys pick a visible cell; the mouse wheel walks the whole list.
pub fn select_slot(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut hotbar: ResMut<Hotbar>,
) {
    let start = hotbar.window_start();
    for (cell, key) in DIGIT_KEYS.iter().enumerate() {
        if keys.just_pressed(*key) {
            hotbar.selected = (start + cell).min(SLOTS.len() - 1);
        }
    }

    let n = SLOTS.len();
    if scroll.delta.y > 0.0 {
        hotbar.selected = (hotbar.selected + n - 1) % n;
    } else if scroll.delta.y < 0.0 {
        hotbar.selected = (hotbar.selected + 1) % n;
    }
}

/// Repaint the bar: which slot each cell shows, and which cell is highlighted.
pub fn update_selection(
    hotbar: Res<Hotbar>,
    mut cells: Query<(&HotbarCell, &mut BorderColor)>,
    mut icons: Query<(&HotbarIcon, &mut ImageNode, &mut Visibility)>,
) {
    if !hotbar.is_changed() {
        return;
    }
    let start = hotbar.window_start();

    for (cell, mut border) in &mut cells {
        *border = if start + cell.0 == hotbar.selected {
            BorderColor::all(Color::WHITE)
        } else {
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.6))
        };
    }

    for (icon, mut image, mut visibility) in &mut icons {
        match SLOTS.get(start + icon.0).copied().flatten() {
            Some(block) => {
                if let Some(atlas) = image.texture_atlas.as_mut() {
                    // Face 4 is a side face — the most recognisable view of a
                    // block as an icon.
                    atlas.index = block_tile(block, 4) as usize;
                }
                *visibility = Visibility::Inherited;
            }
            // The empty hand, or past the end of the list: draw nothing.
            None => *visibility = Visibility::Hidden,
        }
    }
}
