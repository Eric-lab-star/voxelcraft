//! Tab-toggled pause menu with numbered save/load slots, plus a transient
//! on-screen "toast" message (e.g. "Saved to slot 2").

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::chunk::DirtyChunks;
use crate::save::{load_world, save_world, slot_exists, NUM_SLOTS};
use crate::world::World;

const NORMAL: Color = Color::srgb(0.22, 0.22, 0.28);
const HOVER: Color = Color::srgb(0.32, 0.32, 0.42);
const EMPTY: Color = Color::srgb(0.15, 0.15, 0.18);

/// Whether the pause menu is open.
#[derive(Resource, Default)]
pub struct MenuState {
    pub open: bool,
}

/// Run condition: gameplay only ticks while the menu is closed.
pub fn game_active(menu: Res<MenuState>) -> bool {
    !menu.open
}

/// A short-lived on-screen message.
#[derive(Resource, Default)]
pub struct Toast {
    message: String,
    timer: f32,
}

impl Toast {
    pub fn show(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.timer = 2.5;
    }
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct ToastText;

/// A menu button and what it does.
#[derive(Component, Clone, Copy)]
pub enum MenuButton {
    Save(usize),
    Load(usize),
    Quit,
}

pub fn setup_menu(mut commands: Commands) {
    // Toast text, top-centre, hidden until a message is shown.
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                ToastText,
            ));
        });

    // The pause menu overlay.
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(10),
            Visibility::Hidden,
            MenuRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.11, 0.96)),
            ))
            .with_children(|panel| {
                title(panel, "voxelcraft", 34.0);
                title(panel, "Save", 20.0);
                slot_row(panel, true);
                title(panel, "Load", 20.0);
                slot_row(panel, false);
                // Quit
                button(panel, MenuButton::Quit, "Quit", 210.0, NORMAL);
            });
        });
}

fn title(panel: &mut ChildSpawnerCommands, text: &str, size: f32) {
    panel.spawn((
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            margin: UiRect::top(Val::Px(6.0)),
            ..default()
        },
    ));
}

/// A row of per-slot buttons for saving (or loading).
fn slot_row(panel: &mut ChildSpawnerCommands, save: bool) {
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            for slot in 1..=NUM_SLOTS {
                let exists = slot_exists(slot);
                let kind = if save {
                    MenuButton::Save(slot)
                } else {
                    MenuButton::Load(slot)
                };
                // Loading an empty slot is a no-op; show it greyed.
                let bg = if !save && !exists { EMPTY } else { NORMAL };
                let label = format!("{slot}");
                button(row, kind, &label, 66.0, bg);
            }
        });
}

fn button(
    parent: &mut ChildSpawnerCommands,
    kind: MenuButton,
    label: &str,
    width: f32,
    bg: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(width),
                height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            kind,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// Tab toggles the menu open/closed.
pub fn toggle_menu(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<MenuState>) {
    if keys.just_pressed(KeyCode::Tab) {
        menu.open = !menu.open;
    }
}

/// React to open/close: grab or release the cursor and show/hide the panel.
pub fn apply_menu_state(
    menu: Res<MenuState>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut root: Query<&mut Visibility, With<MenuRoot>>,
) {
    if !menu.is_changed() {
        return;
    }
    if let Ok(mut cursor) = cursors.single_mut() {
        if menu.open {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        } else {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    }
    for mut vis in &mut root {
        *vis = if menu.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn menu_button_actions(
    mut buttons: Query<(&Interaction, &MenuButton, &mut BackgroundColor), Changed<Interaction>>,
    mut menu: ResMut<MenuState>,
    mut toast: ResMut<Toast>,
    mut exit: MessageWriter<AppExit>,
    mut world: ResMut<World>,
    mut dirty: ResMut<DirtyChunks>,
) {
    for (interaction, kind, mut bg) in &mut buttons {
        match *interaction {
            Interaction::Pressed => match kind {
                MenuButton::Save(slot) => {
                    if save_world(&world, *slot) {
                        toast.show(format!("Saved to slot {slot}"));
                    }
                    menu.open = false;
                }
                MenuButton::Load(slot) => {
                    if load_world(&mut world, &mut dirty, *slot) {
                        toast.show(format!("Loaded slot {slot}"));
                        menu.open = false;
                    } else {
                        toast.show(format!("Slot {slot} is empty"));
                    }
                }
                MenuButton::Quit => {
                    exit.write(AppExit::Success);
                }
            },
            Interaction::Hovered => *bg = BackgroundColor(HOVER),
            Interaction::None => {
                // Keep empty load-slots visually distinct even when not hovered.
                let base = match kind {
                    MenuButton::Load(slot) if !slot_exists(*slot) => EMPTY,
                    _ => NORMAL,
                };
                *bg = BackgroundColor(base);
            }
        }
    }
}

/// Fade the toast message out over its lifetime.
pub fn update_toast(
    time: Res<Time>,
    mut toast: ResMut<Toast>,
    mut query: Query<(&mut Text, &mut TextColor), With<ToastText>>,
) {
    if toast.timer > 0.0 {
        toast.timer = (toast.timer - time.delta_secs()).max(0.0);
    }
    let alpha = (toast.timer / 0.6).clamp(0.0, 1.0);
    for (mut text, mut color) in &mut query {
        text.0 = toast.message.clone();
        color.0 = Color::srgba(1.0, 1.0, 0.6, alpha);
    }
}
