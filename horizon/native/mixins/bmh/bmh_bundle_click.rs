// EternalBundleItem handles the secondary button in both directions.
use crate::bundle::{Contents, can_be_in_bundle};
use crate::items::{ItemStack, Items};
use std::cell::Cell;

type Refuses = fn(&str) -> bool;
thread_local! {
    static REFUSES: Cell<Option<Refuses>> = const { Cell::new(None) };
}

pub fn set_rule(rule: Option<Refuses>) {
    REFUSES.set(rule);
}

pub enum Action {
    Vanilla,
    NormalPickup,
    Refused,
    IntoCursor,
    IntoSlot,
}

pub fn action(
    items: &Items,
    cursor: Option<&ItemStack>,
    slot: Option<&ItemStack>,
    primary: bool,
    may_modify: impl FnOnce() -> bool,
) -> Action {
    let Some(refuses) = REFUSES.get() else {
        return Action::Vanilla;
    };
    let eternal = |stack: &ItemStack| &*stack.item == "bmh:eternal_bundle";
    let cursor_bundle = cursor.is_some_and(eternal);
    let slot_bundle = slot.is_some_and(eternal);
    if !cursor_bundle && !slot_bundle {
        return Action::Vanilla;
    }
    let cannot_insert = |item: &ItemStack| refuses(&item.item) || !can_be_in_bundle(item);

    // A carried bundle gets the first chance to use the click.
    if cursor_bundle && !primary && cursor.is_some_and(|s| s.count == 1) {
        return match slot {
            None => Action::Vanilla, // Both versions remove one item.
            Some(item) if cannot_insert(item) => Action::Refused,
            Some(_) => Action::IntoCursor,
        };
    }
    if !cursor_bundle
        && primary
        && slot.is_some()
        && cursor.is_some_and(|s| Contents::of(items, s).is_some())
    {
        return Action::Vanilla;
    }

    if slot_bundle && !primary && slot.is_some_and(|s| s.count == 1) && may_modify() {
        return match cursor {
            None => Action::Vanilla,
            Some(item) if cannot_insert(item) => Action::Refused,
            Some(_) => Action::IntoSlot,
        };
    }
    // A normal bundle in the slot can accept the Eternal Bundle from the cursor.
    if !slot_bundle
        && primary
        && cursor_bundle
        && slot.is_some_and(|s| Contents::of(items, s).is_some())
    {
        return Action::IntoSlot;
    }
    Action::NormalPickup
}
