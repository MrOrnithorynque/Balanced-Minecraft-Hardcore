// Module bmh (integrator): the ids that every hook helper uses, and the row of
// BMH in Options > Mods. The ids come from the logic crate, so the game side and
// the tests share one value.

/// The mod id. Each hook helper checks `game.native_on(ID)` first.
pub(super) const ID: &str = bmh_logic::ids::MOD_ID;

/// The mod version, as in gradle.properties.
pub(super) const VERSION: &str = bmh_logic::ids::VERSION;

/// Awards the advancement `bmh:<path>` (`journey/exile`) to the player `name`, through the vanilla
/// command `advancement grant <name> only bmh:<path> done` (verify-1 row 2d; BmhAdvancements.java:31).
/// `run_command` works only where no player's state is swapped in: call it from `server_tick` or a
/// level tick, not from a player's hook. False when the command did not run or the name is not a
/// player name.
pub(super) fn award(game: &mut crate::game::Game<'_>, name: &str, path: &str) -> bool {
    bmh_logic::respawn::award_command(name, path).is_some_and(|line| game.native_run_command(&line))
}

/// The row of BMH in Options > Mods.
pub(super) fn row() -> horizon_client::mod_api::NativeMod {
    horizon_client::mod_api::NativeMod {
        id: ID.into(),
        title: format!("Balanced Minecraft Hardcore {VERSION}"),
        description: "Death sends you on a distant respawn instead of ending the run.".into(),
        keys: Vec::new(),
    }
}
