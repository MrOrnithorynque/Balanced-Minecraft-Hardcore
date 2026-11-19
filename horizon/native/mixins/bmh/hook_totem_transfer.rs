// I_ContainerTotemMixin.java:27-44. Test the same destination that the transfer uses.
fn bmh_totem_transfer(
    game: &crate::game::Game<'_>,
    view: &View,
    stack: &ItemStack,
) -> core::ops::ControlFlow<Option<ItemStack>> {
    use core::ops::ControlFlow;
    if bmh_logic::totem::is_totem(&stack.item) && game.native_on(crate::game::bmh::ID) {
        let containers = game.containers();
        let occupied = (0..view.len()).any(|slot| {
            containers.get(view, slot).is_some_and(|held| bmh_logic::totem::is_totem(&held.item))
        });
        if occupied {
            return ControlFlow::Break(Some(stack.clone()));
        }
    }
    ControlFlow::Continue(())
}
