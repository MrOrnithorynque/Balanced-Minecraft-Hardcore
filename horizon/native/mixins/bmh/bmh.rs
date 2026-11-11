// Module bmh (integrator): the ids that every hook helper uses, and the row of
// BMH in Options > Mods. The ids come from the logic crate, so the game side and
// the tests share one value.

/// The mod id. Each hook helper checks `game.native_on(ID)` first.
pub(super) const ID: &str = bmh_logic::ids::MOD_ID;

/// The mod version, as in gradle.properties.
pub(super) const VERSION: &str = bmh_logic::ids::VERSION;

/// The row of BMH in Options > Mods.
pub(super) fn row() -> horizon_client::mod_api::NativeMod {
    horizon_client::mod_api::NativeMod {
        id: ID.into(),
        title: format!("Balanced Minecraft Hardcore {VERSION}"),
        description: "Death sends you on a distant respawn instead of ending the run.".into(),
        keys: Vec::new(),
    }
}
