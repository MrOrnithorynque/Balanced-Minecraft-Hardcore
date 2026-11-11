// Area settings: the edge between the game and `bmh_logic::settings`.
// It will hold: config file, the six world rules and /bmh rule, the Options > Mods page, the
// Create World button, migration of old world data.
// Owner: the settings agent. The hook helpers (hook_*.rs) call the functions below.

// Contract (docs/horizon/HOOKS.md, "Contracts"): the value of a world rule in the open world.
// Every area reads a rule here, on the server side only. The settings area fills the body and
// keeps this signature. The skeleton answers the value of a new world.
#[allow(dead_code)]
pub(super) fn rule(game: &mut crate::game::Game<'_>, rule: bmh_logic::contract::Rule) -> bool {
    let _ = game;
    rule.default_value()
}

// Contract (docs/horizon/HOOKS.md, "Contracts"): the values of the config file. The settings area
// fills the body and keeps this signature. The skeleton answers the defaults.
#[allow(dead_code)]
pub(super) fn config(game: &mut crate::game::Game<'_>) -> bmh_logic::contract::Config {
    let _ = game;
    bmh_logic::contract::Config::DEFAULT
}

// S2: a click on the Balanced Hardcore button of the Create New World screen.
pub(super) fn on_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = (game, screen, button, modifiers, canceled);
}

// S2: draw the Balanced Hardcore button on the Create New World screen.
pub(super) fn on_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    let _ = (game, batch, screen);
}

// S3: register `config/bmh-common.toml` (min and max distance, max points).
pub(super) fn on_register_configs(
    game: &mut crate::game::Game<'_>,
    configs: &mut Vec<crate::game::native_configs::NativeConfig>,
) {
    let _ = (game, configs);
}

// S3: read the config values at load.
pub(super) fn on_config_loading(
    game: &mut crate::game::Game<'_>,
    config: &crate::game::native_configs::NativeConfigFile,
) {
    let _ = (game, config);
}

// S3: read the config values again after a change.
pub(super) fn on_config_reloading(
    game: &mut crate::game::Game<'_>,
    config: &crate::game::native_configs::NativeConfigFile,
) {
    let _ = (game, config);
}

// S1: register `/bmh rule`.
pub(super) fn on_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    let _ = (game, commands);
}

// S1: run `/bmh rule`.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let _ = (game, player, command, ran);
}

// S4: the buttons of the BMH page in Options > Mods.
pub(super) fn on_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    let _ = (game, mod_id, buttons);
}

// S4: a click on a stepper button of the BMH page.
pub(super) fn on_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    let _ = (game, mod_id, button);
}

// S1, S2: create the world rules at the first open, from the Create New World choice.
pub(super) fn on_world_lifecycle(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::WorldPhase,
    error_text: &str,
) {
    let _ = (game, context, phase, error_text);
}
