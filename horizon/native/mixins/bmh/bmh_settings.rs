// Area settings: the edge between the game and `bmh_logic::settings`.
// It holds the six world rules and `/bmh rule`, the config file `config/bmh-common.toml`, the
// Options > Mods page and the Create New World button. Every decision is in `bmh_logic::settings`;
// this file reads and writes files and the game, and carries out the answers.
// Owner: the settings agent. Log: docs/horizon/log/settings.md.
//
// Where the data lives:
// - The six world rules (S1) are one text file in the world folder, `<world>/bmh/rules.txt`. A
//   level attachment would hold one value for each dimension, and a hook cannot tell which
//   dimension is in place, so a file is the one place that every dimension and every hook sees.
//   The rules are cached in `SERVER_RULES` and the cache follows the world folder. The command
//   `/bmh rule` writes the cache and the file at once.
// - The config (S3) is `config/bmh-common.toml`. The game hands its text to `config_loading` and
//   `config_reloading`. The values are in `COMMON_CONFIG`, one value for the whole process: the
//   window, the integrated server and a dedicated server read the same file.
// - The Create New World choice (S2) is a mod setting, `balanced_hardcore`, in the mod data folder
//   (`CLIENT_CHOICE` caches it for the window). The server reads the same file when it opens a
//   new world for the first time.
//
// A command line `bmh rule ...` is mine. `bmh point ...` belongs to the amulet area: see
// docs/horizon/HOOKS.md, "The /bmh command root".

use bmh_logic::contract::{Config, Rule};
use bmh_logic::settings as logic;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// The rules of the open world and the world folder they belong to.
struct Store {
    dir: Option<PathBuf>,
    rules: logic::Rules,
}

impl Store {
    const EMPTY: Store = Store { dir: None, rules: logic::Rules::DEFAULT };
}

/// The rules of the world that the server has open (server side only).
static SERVER_RULES: Mutex<Store> = Mutex::new(Store::EMPTY);

/// The config values. The game thread of the window, the integrated server and a dedicated server
/// share one file, so they share this value too.
static COMMON_CONFIG: Mutex<Config> = Mutex::new(Config::DEFAULT);

/// The Create New World choice, as the window last read or wrote it.
static CLIENT_CHOICE: Mutex<Option<bool>> = Mutex::new(None);

/// The lock of a static. A panic in another hook leaves the value usable.
fn lock<T>(value: &Mutex<T>) -> MutexGuard<'_, T> {
    value.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------------------------
// The six world rules (S1)
// ---------------------------------------------------------------------------------------------

/// The rules file of a world folder.
fn rules_path(dir: &Path) -> PathBuf {
    dir.join(logic::WORLD_FOLDER).join(logic::RULES_FILE)
}

/// The rules of a world folder: the cache, else the file (and the cache then keeps them). None
/// when the world has no rules file yet.
fn load_rules(dir: &Path) -> Option<logic::Rules> {
    {
        let store = lock(&SERVER_RULES);
        if store.dir.as_deref() == Some(dir) {
            return Some(store.rules);
        }
    }
    let rules = logic::parse_rules(&std::fs::read_to_string(rules_path(dir)).ok()?);
    remember_rules(dir, rules);
    Some(rules)
}

/// Keeps the rules of a world folder in the cache.
fn remember_rules(dir: &Path, rules: logic::Rules) {
    let mut store = lock(&SERVER_RULES);
    store.dir = Some(dir.to_path_buf());
    store.rules = rules;
}

/// Writes the rules file. It writes a temporary file and renames it, so a crash never leaves half a
/// file. A failure is logged: the cache still holds the rules for this run.
fn save_rules(dir: &Path, rules: logic::Rules) {
    let path = rules_path(dir);
    let temporary = path.with_extension("tmp");
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&temporary, logic::rules_text(rules)))
        .and_then(|()| std::fs::rename(&temporary, &path));
    if let Err(error) = written {
        log::warn!("[BalancedMcHardcoreMain] The world rules are not saved to {}: {error}", path.display());
    }
}

// Contract (docs/horizon/HOOKS.md, "Contracts"): the value of a world rule in the open world.
// Every area reads a rule here, on the server side only. A world with no rules file yet answers
// the value of a new world (every rule on), as an unregistered game rule does in Java.
#[allow(dead_code)]
pub(super) fn rule(game: &mut crate::game::Game<'_>, rule: bmh_logic::contract::Rule) -> bool {
    let Some(dir) = game.native_world_dir() else {
        return rule.default_value();
    };
    load_rules(&dir).unwrap_or(logic::Rules::DEFAULT).get(rule)
}

// Contract (docs/horizon/HOOKS.md, "Contracts"): the values of the config file. The values are
// the defaults until the game hands over the text of the file at its start.
#[allow(dead_code)]
pub(super) fn config(game: &mut crate::game::Game<'_>) -> bmh_logic::contract::Config {
    let _ = game;
    *lock(&COMMON_CONFIG)
}

/// A rule changed by the command: the cache and the file take it at once.
fn set_rule(game: &crate::game::Game<'_>, rule: Rule, value: bool) -> bool {
    let Some(dir) = game.native_world_dir() else {
        return false;
    };
    let rules = load_rules(&dir).unwrap_or(logic::Rules::DEFAULT).with(rule, value);
    remember_rules(&dir, rules);
    save_rules(&dir, rules);
    true
}

/// The line "Game rule bmh:x is currently set to: true" of vanilla's `/gamerule`.
fn query_text(rule: Rule, value: bool) -> horizon_client::text::Text {
    use horizon_client::text::Text;
    Text::translate(
        logic::KEY_QUERY,
        &[Text::literal(logic::rule_id(rule)), Text::literal(logic::bool_text(value))],
    )
}

// S1: register `/bmh rule`. The amulet area registers `bmh point ...` in its own function: the
// server merges the usages that start with the same word.
pub(super) fn on_register_commands(
    game: &mut crate::game::Game<'_>,
    commands: &mut Vec<horizon_client::mod_api::NativeCommand>,
) {
    let _ = game;
    for usage in logic::COMMAND_USAGES {
        commands.push(horizon_client::mod_api::NativeCommand::op(usage, logic::COMMAND_LEVEL));
    }
}

// S1: run `/bmh rule`. A line of another area (`bmh point ...`) is left alone.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    use horizon_client::text::{self, Text};
    let _ = player;
    if *ran {
        return;
    }
    let parsed = logic::parse_rule_command(command);
    if parsed == logic::RuleCommand::NotMine {
        return;
    }
    *ran = true;
    match parsed {
        logic::RuleCommand::NotMine => {}
        logic::RuleCommand::List => {
            for rule in Rule::ALL {
                let value = self::rule(game, rule);
                game.native_feedback(query_text(rule, value));
            }
        }
        logic::RuleCommand::Query(rule) => {
            let value = self::rule(game, rule);
            game.native_feedback(query_text(rule, value));
        }
        logic::RuleCommand::Set(rule, value) => {
            if set_rule(game, rule, value) {
                let line = Text::translate(
                    logic::KEY_SET,
                    &[Text::literal(logic::rule_id(rule)), Text::literal(logic::bool_text(value))],
                );
                game.native_feedback(line);
            } else {
                log::warn!("[BalancedMcHardcoreMain] /bmh rule found no open world folder");
            }
        }
        logic::RuleCommand::UnknownRule(name) => {
            game.native_feedback(Text::tr(logic::KEY_UNKNOWN, &[&name]).with_color(text::RED));
        }
        logic::RuleCommand::BadValue(word) => {
            game.native_feedback(Text::tr(logic::KEY_BAD_VALUE, &[&word]).with_color(text::RED));
        }
        logic::RuleCommand::Usage => {
            game.native_feedback(Text::translate(logic::KEY_USAGE, &[]).with_color(text::RED));
        }
    }
}

/// The rules of a Java world, read from its game rules saved data before Horizon's first save
/// drops them (verify-5 G4). None when the file is missing or holds no BMH rule.
fn import_java_rules(dir: &Path) -> Option<logic::Rules> {
    use horizon_world::nbt::Tag;
    let file = dir.join("data").join("minecraft").join("game_rules.dat");
    let tag = horizon_client::save::read_dat(&file)?;
    let data = tag.get("data")?;
    logic::import_rules(|key| match data.get(key) {
        Some(Tag::String(text)) => Some(logic::Saved::Text(text.to_string_lossy())),
        Some(Tag::Byte(byte)) => Some(logic::Saved::Byte(*byte)),
        Some(_) => Some(logic::Saved::Other),
        None => None,
    })
}

/// True when the world has not been played: its game time is 0. A world whose time the game cannot
/// tell counts as played, so the Create New World choice never reaches an old world by mistake.
fn is_new_world(game: &crate::game::Game<'_>, dir: &Path) -> bool {
    let time = game
        .native_level(bmh_logic::respawn::OVERWORLD)
        .and_then(|level| game.native_level_game_time(level))
        .or_else(|| horizon_client::save::Level::read(dir).map(|level| level.time));
    time == Some(0)
}

// S1, S2: create the world rules at the first open, from the Java world or from the Create New
// World choice. At the close the cache is emptied, so the next world reads its own file.
pub(super) fn on_world_lifecycle(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::WorldPhase,
    error_text: &str,
) {
    use horizon_client::mod_api::WorldPhase;
    let _ = (context, error_text);
    match phase {
        WorldPhase::WorldOpenReady => open_world(game),
        WorldPhase::WorldCloseFinal | WorldPhase::WorldCloseFailed => {
            *lock(&SERVER_RULES) = Store::EMPTY;
        }
        _ => {}
    }
}

/// The first open of a world with the mod writes its rules file. A world that has the file keeps it.
fn open_world(game: &mut crate::game::Game<'_>) {
    let Some(dir) = game.native_world_dir() else {
        return;
    };
    let file = std::fs::read_to_string(rules_path(&dir)).ok().map(|text| logic::parse_rules(&text));
    let first_open = file.is_none();
    let (imported, new_world, choice) = if first_open {
        let choice = game
            .native_setting(crate::game::bmh::ID, logic::CHOICE_SETTING)
            .and_then(|text| logic::parse_choice(&text));
        (import_java_rules(&dir), is_new_world(game, &dir), choice)
    } else {
        (None, false, None)
    };
    let rules = logic::initial_rules(file, imported, new_world, choice);
    remember_rules(&dir, rules);
    if first_open {
        save_rules(&dir, rules);
        log::info!(
            "[BalancedMcHardcoreMain] World rules created: java world = {}, new world = {new_world}, choice = {choice:?}",
            imported.is_some()
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The Create New World button (S2)
// ---------------------------------------------------------------------------------------------

/// The choice of the window: on by default, as the Java rules are.
fn create_choice(game: &crate::game::Game<'_>) -> bool {
    if let Some(choice) = *lock(&CLIENT_CHOICE) {
        return choice;
    }
    let choice = game
        .native_setting(crate::game::bmh::ID, logic::CHOICE_SETTING)
        .and_then(|text| logic::parse_choice(&text))
        .unwrap_or(true);
    *lock(&CLIENT_CHOICE) = Some(choice);
    choice
}

// S2: a click on the Balanced Hardcore button of the Create New World screen.
pub(super) fn on_screen_mouse_button_pressed_pre(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = modifiers;
    if *canceled || button != 0 || screen.id != logic::CREATE_SCREEN {
        return;
    }
    if !logic::inside(logic::button_rect(screen.rect), screen.mouse) {
        return;
    }
    let choice = !create_choice(game);
    *lock(&CLIENT_CHOICE) = Some(choice);
    game.native_set_setting(crate::game::bmh::ID, logic::CHOICE_SETTING, logic::bool_text(choice));
    *canceled = true;
}

// S2: draw the Balanced Hardcore button on the Create New World screen.
pub(super) fn on_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use horizon_client::text::Text;
    if screen.id != logic::CREATE_SCREEN {
        return;
    }
    let rect = logic::button_rect(screen.rect);
    let hovered = logic::inside(rect, screen.mouse);
    let sprite = if hovered {
        "gui/sprites/widget/button_highlighted"
    } else {
        "gui/sprites/widget/button"
    };
    // The fill shows if the sprite is missing; the sprite covers it otherwise.
    game.native_fill(batch, rect, 0xFF30_3030);
    game.native_gui_nine_slice(batch, sprite, rect, 0xFFFF_FFFF);
    let state = if create_choice(game) { "options.on" } else { "options.off" };
    let label = Text::option_name_value(
        Text::translate("bmh.createWorld.balancedHardcore", &[]),
        Text::translate(state, &[]),
    )
    .plain();
    let width = game.native_text_width(&label);
    let x = rect[0] + ((rect[2] - width) / 2.0).floor();
    let y = rect[1] + (rect[3] - 8.0) / 2.0;
    game.native_text(batch, &label, [x, y], 0xFF_FFFF, true);
    if hovered {
        let info = Text::translate("bmh.createWorld.balancedHardcore.info", &[]);
        game.native_gui_tooltip(batch, &[info], screen.mouse);
    }
}

// ---------------------------------------------------------------------------------------------
// The config file (S3) and its page (S4)
// ---------------------------------------------------------------------------------------------

// S3: register `config/bmh-common.toml` (min and max distance, max points).
pub(super) fn on_register_configs(
    game: &mut crate::game::Game<'_>,
    configs: &mut Vec<crate::game::native_configs::NativeConfig>,
) {
    let _ = game;
    configs.push(crate::game::native_configs::NativeConfig::new(
        crate::game::bmh::ID,
        crate::game::native_configs::NativeConfigKind::Common,
        &logic::config_text(Config::DEFAULT),
    ));
}

/// Takes the values of the config text the game read.
fn apply_config(config: &crate::game::native_configs::NativeConfigFile) {
    if config.mod_id != crate::game::bmh::ID
        || config.kind != crate::game::native_configs::NativeConfigKind::Common
    {
        return;
    }
    *lock(&COMMON_CONFIG) = logic::parse_config(&config.text);
}

// S3: read the config values at load.
pub(super) fn on_config_loading(
    game: &mut crate::game::Game<'_>,
    config: &crate::game::native_configs::NativeConfigFile,
) {
    let _ = game;
    apply_config(config);
}

// S3: read the config values again after a change.
pub(super) fn on_config_reloading(
    game: &mut crate::game::Game<'_>,
    config: &crate::game::native_configs::NativeConfigFile,
) {
    let _ = game;
    apply_config(config);
}

// S4: the buttons of the BMH page in Options > Mods. For each value there is a button that raises
// it and a button that lowers it, each with the value in its label. The game asks again after each
// click, so the labels follow the value.
pub(super) fn on_settings(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use horizon_client::text::Text;
    let _ = game;
    if mod_id != crate::game::bmh::ID {
        return;
    }
    let config = *lock(&COMMON_CONFIG);
    for (field, raise) in logic::buttons() {
        let name = Text::translate(logic::field_key(*field), &[]).plain();
        let value = logic::field_value(config, *field).to_string();
        let label = Text::tr(logic::KEY_STEP, &[&logic::sign(*raise), &name, &value]);
        buttons.push(label.plain());
    }
}

// S4: a click on a stepper button of the BMH page. The new values go to the config file at once and
// to the cache. The game reads the file again within a second and gives the same values back.
pub(super) fn on_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    let _ = game;
    if mod_id != crate::game::bmh::ID {
        return;
    }
    let Some((field, raise)) = logic::button(button) else {
        return;
    };
    let current = *lock(&COMMON_CONFIG);
    let next = logic::step(current, field, raise);
    if next == current {
        return;
    }
    *lock(&COMMON_CONFIG) = next;
    let written = crate::game::Game::native_config_dir()
        .map_err(|error| error.to_string())
        .and_then(|dir| {
            std::fs::write(dir.join(logic::CONFIG_FILE), logic::config_text(next))
                .map_err(|error| error.to_string())
        });
    if let Err(error) = written {
        log::warn!("[BalancedMcHardcoreMain] The config file is not saved: {error}");
    }
}
