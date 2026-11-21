fn bmh_ritual_model(
    mut state: ItemRenderState,
    game: &Game<'_>,
    stack: &ItemStack,
    owner: Option<&OwnerState>,
) -> ItemRenderState {
    if &*stack.item == bmh_logic::ids::SLEEPING_TOTEM
        && game.native_on(crate::game::bmh::ID)
        && owner.is_some_and(|p| p.using_this_item && p.entity_type == "minecraft:player")
    {
        for layer in &mut state.layers {
            layer.foil = horizon_assets::item_model::FoilType::Standard;
        }
        if matches!(state.display, DisplayContext::FirstPersonLeftHand | DisplayContext::FirstPersonRightHand) {
            state.hdr_light = 15;
        }
    }
    state
}
