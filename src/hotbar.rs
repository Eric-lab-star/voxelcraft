//! A Minecraft-style hotbar: a row of block slots at the bottom of the screen.
//! Pick the active block with number keys 1–7 or the mouse wheel; the selected
//! block is what right-click places.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::block::Block;
use crate::texture::{block_tile, BlockAtlas};

/// The blocks available on the hotbar, left to right.
pub const SLOTS: [Block; 7] = [
    Block::Grass,
    Block::Dirt,
    Block::Stone,
    Block::Sand,
    Block::Wood,
    Block::Leaves,
    Block::Water,
];

/// Digit keys mapped to slot indices, in order.
const DIGIT_KEYS: [KeyCode; 7] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
];

/// Which hotbar slot is currently selected.
#[derive(Resource, Default)]
pub struct Hotbar {
    pub selected: usize,
}

impl Hotbar {
    /// The block the player will place.
    pub fn block(&self) -> Block {
        SLOTS[self.selected]
    }
}

/// Marks a slot cell so we can recolour its border when the selection changes.
#[derive(Component)]
pub struct HotbarSlot(usize);

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
            // The bar itself.
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
                for (i, block) in SLOTS.iter().enumerate() {
                    // Show the block's side face as the slot icon.
                    let index = block_tile(*block, 4) as usize;
                    bar.spawn((
                        Node {
                            width: Val::Px(46.0),
                            height: Val::Px(46.0),
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
                        HotbarSlot(i),
                    ))
                    .with_children(|slot| {
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
                                    index,
                                },
                            ),
                        ));
                    });
                }
            });
        });
}

/// Number keys pick a slot directly; the mouse wheel cycles through slots.
pub fn select_slot(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut hotbar: ResMut<Hotbar>,
) {
    for (i, key) in DIGIT_KEYS.iter().enumerate() {
        if keys.just_pressed(*key) {
            hotbar.selected = i;
        }
    }

    let n = SLOTS.len();
    if scroll.delta.y > 0.0 {
        hotbar.selected = (hotbar.selected + n - 1) % n;
    } else if scroll.delta.y < 0.0 {
        hotbar.selected = (hotbar.selected + 1) % n;
    }
}

/// Highlight the selected slot's border (only when the selection changed).
pub fn update_selection(hotbar: Res<Hotbar>, mut slots: Query<(&HotbarSlot, &mut BorderColor)>) {
    if !hotbar.is_changed() {
        return;
    }
    for (slot, mut border) in &mut slots {
        *border = if slot.0 == hotbar.selected {
            BorderColor::all(Color::WHITE)
        } else {
            BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.6))
        };
    }
}
