fn bmh_bundle_menu(
    menu: &mut Menu,
    env: &Env,
    slot: usize,
    primary: bool,
) -> std::ops::ControlFlow<bool> {
    use crate::bmh_bundle_click::{self, Action};
    use std::ops::ControlFlow;
    let decision = bmh_bundle_click::action(
        env.items,
        menu.carried.as_ref(),
        menu.slot(slot),
        primary,
        || menu.allow_modification(env, slot),
    );
    let sounds = menu.player_sounds.len();
    let silent_failure = match decision {
        Action::Vanilla => return ControlFlow::Continue(()),
        Action::NormalPickup => return ControlFlow::Break(false),
        Action::Refused => return ControlFlow::Break(true),
        Action::IntoCursor => {
            menu.carried_bundle_click(env, slot, true);
            true
        }
        Action::IntoSlot => {
            let eternal = menu
                .slot(slot)
                .is_some_and(|s| &*s.item == "bmh:eternal_bundle");
            menu.clicked_bundle_click(env, slot, true);
            eternal
        }
    };
    if silent_failure && menu.player_sounds.get(sounds) == Some(&INSERT_FAIL) {
        menu.player_sounds.truncate(sounds);
    }
    menu.inventory_slots_changed(env);
    ControlFlow::Break(true)
}
