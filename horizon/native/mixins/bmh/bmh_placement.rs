// The two menu engines both ask the destination slot before they change a stack.
use super::{GRID, Input, ItemStack, Kind, MAIN, Menu, PLAYER, RESULT};
use crate::lan::menu::{Slots, View, input};
use std::cell::Cell;

type SlotId = (usize, usize);
type Drag = Option<(u8, Vec<usize>)>;

// Rust clones the moving stack. Use its backing slot to recognize moves within one inventory.
#[derive(Clone, Copy)]
enum Movement {
    Cursor,
    Shift(SlotId),
    Swap { hotbar: SlotId, clicked: SlotId },
    DragRelease,
}

impl Movement {
    fn source(self, target: SlotId) -> Option<SlotId> {
        match self {
            Self::Shift(source) => Some(source),
            Self::Swap { hotbar, clicked } if target == hotbar => Some(clicked),
            Self::Swap { hotbar, .. } => Some(hotbar),
            Self::Cursor | Self::DragRelease => None,
        }
    }
}

thread_local! {
    // Each Game sets its own thread's option before a server click or client prediction.
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static MOVEMENT: Cell<Movement> = const { Cell::new(Movement::Cursor) };
}

pub fn set_enabled(enabled: bool) {
    ENABLED.set(enabled);
    MOVEMENT.set(Movement::Cursor);
}

fn active(stack: &ItemStack) -> bool {
    stack.count > 0 && &*stack.item == "bmh:totem_of_eternity"
}

fn contains_other<'a>(
    target: SlotId,
    source: Option<SlotId>,
    mut slots: impl Iterator<Item = (usize, Option<&'a ItemStack>)>,
) -> bool {
    slots.any(|(i, stack)| {
        i != target.1 && Some((target.0, i)) != source && stack.is_some_and(active)
    })
}

fn number_key(button: i32) -> Option<SlotId> {
    matches!(button, 0..=8 | 40).then_some((PLAYER, button as usize))
}

pub(super) fn menu_start(menu: &Menu, slot: i32, button: i32, input: Input) -> bool {
    MOVEMENT.set(Movement::Cursor);
    if !ENABLED.get() || menu.kind == Kind::ItemPicker || menu.quick_crafting() {
        return false;
    }
    let Some(clicked) = usize::try_from(slot).ok().and_then(|i| menu.slots.get(i)) else {
        return false;
    };
    let clicked = (clicked.container, clicked.index);
    if input == Input::QuickMove {
        MOVEMENT.set(Movement::Shift(clicked));
    } else if input == Input::Swap
        && let Some(hotbar) = number_key(button)
    {
        MOVEMENT.set(Movement::Swap { hotbar, clicked });
        // The engine writes this side of a number-key swap without asking its slot.
        return menu.containers[clicked.0][clicked.1]
            .as_ref()
            .is_some_and(active)
            && contains_other(
                hotbar,
                Some(clicked),
                menu.containers[PLAYER]
                    .iter()
                    .map(Option::as_ref)
                    .enumerate(),
            );
    }
    false
}

pub(super) fn menu_refuses(menu: &Menu, slot: usize, stack: &ItemStack) -> bool {
    if !ENABLED.get() || !active(stack) || menu.kind == Kind::ItemPicker {
        return false;
    }
    let slot = menu.slots[slot];
    if matches!(menu.kind, Kind::Mod(_)) && slot.container != PLAYER {
        return false;
    }
    contains_other(
        (slot.container, slot.index),
        MOVEMENT.get().source((slot.container, slot.index)),
        menu.containers[slot.container]
            .iter()
            .map(Option::as_ref)
            .enumerate(),
    )
}

pub(super) fn menu_end() {
    MOVEMENT.set(Movement::Cursor);
}

fn view_slot(view: &View<'_>, slot: usize) -> SlotId {
    if let Some(items) = &view.container {
        if slot < items.len() {
            return (MAIN, slot);
        }
        let player = slot - items.len();
        return (PLAYER, if player < 27 { player + 9 } else { player - 27 });
    }
    match slot {
        0 => (RESULT, 0),
        1..=4 => (GRID, slot - 1),
        5..=8 => (PLAYER, 44 - slot),
        9..=35 => (PLAYER, slot),
        36..=44 => (PLAYER, slot - 36),
        _ => (PLAYER, 40),
    }
}

fn view_contains_other(view: &View<'_>, target: SlotId, source: Option<SlotId>) -> bool {
    let guest = &view.guest;
    match target.0 {
        PLAYER => contains_other(
            target,
            source,
            guest
                .inventory
                .iter()
                .take(36)
                .map(Option::as_ref)
                .chain(guest.armor.iter().rev().map(Option::as_ref))
                .chain([guest.offhand.as_ref()])
                .enumerate(),
        ),
        MAIN if !guest
            .menu
            .as_ref()
            .is_some_and(|m| m.slots == Slots::Native || m.mod_type.is_some()) =>
        {
            view.container.as_deref().is_some_and(|slots| {
                contains_other(target, source, slots.iter().map(Option::as_ref).enumerate())
            })
        }
        GRID => contains_other(
            target,
            source,
            guest.craft_grid.iter().map(Option::as_ref).enumerate(),
        ),
        _ => false,
    }
}

pub fn view_refuses(view: &View<'_>, slot: usize, stack: &ItemStack) -> bool {
    if !ENABLED.get() || !active(stack) {
        return false;
    }
    let target = view_slot(view, slot);
    view_contains_other(view, target, MOVEMENT.get().source(target))
}

pub fn view_start(view: &View<'_>, drag: &mut Drag, slot: i32, button: i32, mode: i32) -> bool {
    MOVEMENT.set(Movement::Cursor);
    if !ENABLED.get() {
        return false;
    }
    let clicked = usize::try_from(slot).ok().filter(|&i| i < view.len());
    if mode == input::QUICK_MOVE {
        if let Some(i) = clicked {
            MOVEMENT.set(Movement::Shift(view_slot(view, i)));
        }
    } else if mode == input::SWAP
        && let (Some(i), Some(hotbar)) = (clicked, number_key(button))
    {
        let clicked = view_slot(view, i);
        MOVEMENT.set(Movement::Swap { hotbar, clicked });
        if view.get(i).as_ref().is_some_and(active)
            && view_contains_other(view, hotbar, Some(clicked))
        {
            *drag = None;
            return true;
        }
    } else if mode == input::QUICK_CRAFT
        && let Some(carried) = view.guest.carried.as_ref().filter(|s| active(s))
        && let Some((kind, slots)) = drag
    {
        match button & 3 {
            // A survival drag cannot select more slots than it has items.
            1 if *kind != 2 && slots.len() >= carried.count as usize => return true,
            2 | 3 if slots.len() > 1 => MOVEMENT.set(Movement::DragRelease),
            _ => {}
        }
    }
    false
}

/// The guest chest engine checks capacity again as it writes each drag slot.
pub fn drag_refuses(view: &View<'_>, slot: usize, stack: &ItemStack) -> bool {
    matches!(MOVEMENT.get(), Movement::DragRelease)
        && ENABLED.get()
        && active(stack)
        && (view_contains_other(view, view_slot(view, slot), None)
            || view
                .get(slot)
                .as_ref()
                .is_some_and(|held| !held.same_item_same_components(stack)))
}
