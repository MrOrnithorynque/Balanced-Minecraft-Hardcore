// Hook `mods`: the row of BMH in Options > Mods. It shows even when the mod is off,
// so the player can turn it on again.
fn bmh_hook_mods(mods: &mut Vec<horizon_client::mod_api::NativeMod>) {
    mods.push(crate::game::bmh::row());
}
