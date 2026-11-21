fn bmh_bundle_view(
    view: &mut View<'_>,
    slot: usize,
    right: bool,
    rules: &dyn Rules,
) -> std::ops::ControlFlow<bool> {
    use crate::bmh_bundle_click::{self, Action};
    use std::ops::ControlFlow;
    let clicked = view.get(slot);
    let decision = bmh_bundle_click::action(
        rules.items(),
        view.guest.carried.as_ref(),
        clicked.as_ref(),
        !right,
        || {
            view.may_pickup(slot, rules)
                && clicked
                    .as_ref()
                    .is_none_or(|s| view.may_place(slot, s, rules))
        },
    );
    match decision {
        Action::Vanilla => return ControlFlow::Continue(()),
        Action::NormalPickup => return ControlFlow::Break(false),
        Action::Refused => {}
        Action::IntoCursor => {
            if view.may_pickup(slot, rules)
                && let (Some(bundle), Some(mut input)) = (view.guest.carried.as_mut(), clicked)
                && let Some(mut contents) = Contents::of(rules.items(), bundle)
            {
                contents.try_insert(rules.items(), &mut input);
                contents.store(bundle);
                view.set(slot, (input.count > 0).then_some(input));
            }
        }
        Action::IntoSlot => {
            if let Some(mut bundle) = clicked
                && view.may_pickup(slot, rules)
                && view.may_place(slot, &bundle, rules)
                && let Some(mut contents) = Contents::of(rules.items(), &bundle)
                && let Some(mut input) = view.guest.carried.take()
            {
                contents.try_insert(rules.items(), &mut input);
                contents.store(&mut bundle);
                view.guest.carried = (input.count > 0).then_some(input);
                view.set(slot, Some(bundle));
            }
        }
    }
    ControlFlow::Break(true)
}
