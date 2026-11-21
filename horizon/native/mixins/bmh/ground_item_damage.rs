fn bmh_ground_item_damage(item: &ItemEntity) -> core::ops::ControlFlow<Option<Vec<ItemStack>>> {
    if crate::bmh_ground_items::protected(&item.stack.item) {
        return core::ops::ControlFlow::Break(None);
    }
    core::ops::ControlFlow::Continue(())
}
