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
// - The world creation screen stores its six rules in the `balanced_hardcore` mod setting.
//   The server reads this setting when it opens the new world for the first time.
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

/// The Java rules read when the Overworld opened: the world folder and what the game rules file held
/// (None when it held no BMH rule). A dedicated server saves the world before `WorldOpenReady`, and
/// that save drops the keys of the Java mod, so the file is read earlier, at `level_load`.
static PENDING_IMPORT: Mutex<Option<(PathBuf, Option<logic::Rules>)>> = Mutex::new(None);

/// The `bmh rule ...` lines that typed `/gamerule bmh:...` commands asked for, with the player who
/// typed each one. `player_command` cancels the typed line and cannot tell the player's level, so
/// the next `server_tick` runs the converted line as that player (`native_run_command_as`), and
/// `/bmh rule` refuses a player below the level of `/gamerule` as it does for a typed `/bmh rule`.
static SERVER_GAMERULE_QUEUE: Mutex<Vec<(u128, String)>> = Mutex::new(Vec::new());

/// The most lines kept between two ticks, so a flood of commands cannot grow the queue.
const GAMERULE_QUEUE_LIMIT: usize = 64;

/// The config values. The game thread of the window, the integrated server and a dedicated server
/// share one file, so they share this value too.
static COMMON_CONFIG: Mutex<Config> = Mutex::new(Config::DEFAULT);

/// The Create New World choice of the screen the window shows (or showed last).
static CLIENT_CHOICE: Mutex<logic::Rules> = Mutex::new(logic::Rules::DEFAULT);

/// True while the title held one of the Create New World screens at the last screen hook. A Create
/// New World screen that the title shows after another state is a new one, and its choice starts ON.
static CREATE_OPEN: Mutex<bool> = Mutex::new(false);

/// The folder of the world that the world list had selected at the last screen hook. Re-Create opens
/// the Create New World screen from that row, and the screen does not tell which world it copies.
static LAST_SELECTED: Mutex<Option<PathBuf>> = Mutex::new(None);

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

// S1: run `/bmh rule`. A line of another area (`bmh point ...`) is left alone. Horizon checks one
// operator level for the whole `/bmh` root (the lowest of its usages, and `/bmh point` is level 0),
// so this area checks the level of `/bmh rule` itself.
pub(super) fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &horizon_client::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    run_rule_command(game, Some(player.permission), command, ran);
}

// S1: `/bmh rule` typed on the console of a dedicated server or sent through RCON. The console is
// level 4, so it needs no level check.
pub(super) fn on_perform_console_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    run_rule_command(game, None, command, ran);
}

// S1: hear every command a player types (API 95). `/gamerule bmh:<name> [value]` is the Java
// command for a BMH rule. Horizon's own `/gamerule` knows vanilla rules only, so the line is stopped
// here and the next `server_tick` runs it as `/bmh rule <name> [value]` for the same player. A vanilla
// rule (`/gamerule keep_inventory true`) is left alone. The answer text is vanilla's own.
pub(super) fn on_player_command(
    game: &mut crate::game::Game<'_>,
    player: u128,
    name: &str,
    command: &str,
    canceled: &mut bool,
    replies: &mut Vec<horizon_client::text::Text>,
) {
    let _ = (game, name, replies);
    if *canceled {
        return;
    }
    let Some(line) = logic::gamerule_line_to_bmh(command) else {
        return;
    };
    *canceled = true;
    log::info!("[BalancedMcHardcoreMain] /gamerule bmh:... taken from a player, it runs as: {line}");
    let mut queue = lock(&SERVER_GAMERULE_QUEUE);
    if queue.len() < GAMERULE_QUEUE_LIMIT {
        queue.push((player, line));
    }
}

// S1: run the `/gamerule bmh:...` lines of the tick as the players who typed them. `run_command`
// works here: no player is swapped in. The answer reaches the player as the answer to their own
// command.
pub(super) fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    context: &horizon_client::mod_api::NativeLifecycleContext,
    phase: horizon_client::mod_api::ServerTickPhase,
) {
    let _ = context;
    let horizon_client::mod_api::ServerTickPhase::ServerTickEnd { .. } = phase else {
        return;
    };
    let queued = std::mem::take(&mut *lock(&SERVER_GAMERULE_QUEUE));
    for (player, line) in queued {
        let result = game.native_run_command_as(player, &line);
        if !result.ran {
            log::warn!("[BalancedMcHardcoreMain] /gamerule bmh:... ran nothing for the line: {line}");
        }
    }
}

// S1: complete the rule name of `/bmh rule <name>` (the asked-server kind `word?`). `arguments` are
// the words after `bmh`, the last one the word being typed.
pub(super) fn on_tab_complete_v2(
    game: &mut crate::game::Game<'_>,
    label: &str,
    arguments: &[String],
    range: [usize; 2],
    suggestions: &mut Vec<horizon_client::mod_api::Suggestion>,
) {
    let _ = game;
    if label != "bmh" || arguments.len() != 2 || arguments[0] != "rule" {
        return;
    }
    for name in logic::rule_name_matches(&arguments[1]) {
        suggestions.push(horizon_client::mod_api::Suggestion {
            range: (range[0], range[1]),
            text: name.to_owned(),
            tooltip: None,
            int_value: None,
        });
    }
}

/// `permission` is the operator level of the player, None for the console.
fn run_rule_command(
    game: &mut crate::game::Game<'_>,
    permission: Option<u8>,
    command: &str,
    ran: &mut bool,
) {
    use horizon_client::text::{self, Text};
    if *ran {
        return;
    }
    let parsed = logic::parse_rule_command(command);
    if parsed == logic::RuleCommand::NotMine {
        return;
    }
    *ran = true;
    // The line that refuses a player below the level of `/gamerule`: vanilla's unknown command.
    if permission.is_some_and(|level| !logic::may_run_rule(level)) {
        game.native_feedback(Text::translate("command.unknown.command", &[]).with_color(text::RED));
        return;
    }
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
/// drops them (verify-5 G4). None when the file is missing or holds no BMH rule. The single-player
/// server saves after `WorldOpenReady`; a dedicated server saves before it, so `on_level_load`
/// calls this.
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

// S1: the Overworld opens. A world with no rules file reads the rules of the Java mod now, before
// the dedicated server's first save drops them. The world folder is set when this fires
// (game.rs:12453 sets `world_root`, :12621 fires `native_level_opened`; dedicated.rs:1253 saves later).
pub(super) fn on_level_load(game: &mut crate::game::Game<'_>, dimension: &str) {
    if dimension != bmh_logic::respawn::OVERWORLD {
        return;
    }
    let Some(dir) = game.native_world_dir() else {
        return;
    };
    if rules_path(&dir).exists() {
        *lock(&PENDING_IMPORT) = None;
        return;
    }
    let imported = import_java_rules(&dir);
    *lock(&PENDING_IMPORT) = Some((dir, imported));
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
            *lock(&PENDING_IMPORT) = None;
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
        let pending = lock(&PENDING_IMPORT).take();
        let imported = match pending {
            Some((pending_dir, imported)) if pending_dir == dir => imported,
            _ => import_java_rules(&dir),
        };
        (imported, is_new_world(game, &dir), choice)
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

/// The choice of the screen: ON until the player flips it.
fn create_choice() -> bool {
    lock(&CLIENT_CHOICE).any()
}

fn set_create_rules(game: &crate::game::Game<'_>, rules: logic::Rules) {
    *lock(&CLIENT_CHOICE) = rules;
    game.native_set_setting(crate::game::bmh::ID, logic::CHOICE_SETTING, &logic::choice_text(rules));
}

/// Start each creation screen with the selected source rules or the defaults for a new world.
/// Keep that choice when the player opens a preset editor or the Data Packs screen.
///
/// This reads the title's own state, not the id of the screen that fired the hook: Data Packs opens
/// a screen of the game's own over the title (`Action::DataPacks` sets `Screen::Options`), and that
/// screen fires the screen hooks with its own id while the title still holds the Create New World
/// screen. With the hook's id, the way back from Data Packs put the choice back to ON (check k5e).
fn track_screen(game: &crate::game::Game<'_>) {
    if let Some(title) = game.title.as_ref()
        && let crate::title::Mode::Select { worlds, selected: Some(row) } = &title.mode
        && let Some(world) = worlds.get(*row)
    {
        *lock(&LAST_SELECTED) = Some(title.saves.join(&world.folder));
    }
    let in_create = game.title.as_ref().is_some_and(|title| logic::is_create_screen(title.mode.id()));
    let was_open = std::mem::replace(&mut *lock(&CREATE_OPEN), in_create);
    if in_create && !was_open {
        set_create_rules(game, recreate_rules(game).unwrap_or(logic::Rules::DEFAULT));
    }
}

/// Re-Create copies the selected world's rules. The initial Create screen has `commands` set only for Re-Create.
fn recreate_rules(game: &crate::game::Game<'_>) -> Option<logic::Rules> {
    let title = game.title.as_ref()?;
    let crate::title::Mode::Create { options, .. } = &title.mode else {
        return None;
    };
    options.commands?;
    let dir = lock(&LAST_SELECTED).clone()?;
    std::fs::read_to_string(rules_path(&dir)).ok().map(|text| logic::parse_rules(&text))
        .or_else(|| import_java_rules(&dir))
}

/// The Java button is greyed out in Hardcore and in a debug world, where nobody respawns and BMH has
/// nothing to do (CreateWorldGameTabMixin.java:49-51). The choice keeps its value.
fn create_locked(game: &crate::game::Game<'_>) -> bool {
    let Some(title) = game.title.as_ref() else {
        return false;
    };
    match &title.mode {
        crate::title::Mode::Create { options, .. } => {
            options.hardcore || options.world_type == crate::save::WorldPreset::Debug
        }
        _ => false,
    }
}

/// The tab that the Create New World screen shows (0 Game, 1 World, 2 More). `NativeScreenView`
/// has no tab, so this reads the title's state: `Game.title` is private to `game.rs`, and this
/// module sits inside it. None when the title shows another screen, and while the title prepares
/// the world (it takes no click then).
fn create_tab(game: &crate::game::Game<'_>) -> Option<usize> {
    let title = game.title.as_ref()?;
    match &title.mode {
        crate::title::Mode::Create { options, .. } if !title.preparing() => Some(options.tab),
        _ => None,
    }
}

/// The button of the screen, when it is drawn and takes clicks: on the Create New World screen, in
/// the Game tab, under Allow Commands.
fn button_rect(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) -> Option<[f32; 4]> {
    if screen.id != logic::CREATE_SCREEN {
        return None;
    }
    logic::button_rect(screen.rect, create_tab(game)?)
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
    track_screen(game);
    if *canceled || button != 0 {
        return;
    }
    let Some(rect) = button_rect(game, screen) else {
        return;
    };
    if !logic::inside(rect, screen.mouse) {
        return;
    }
    if create_locked(game) {
        *canceled = true;
        return;
    }
    set_create_rules(game, logic::Rules::all(!create_choice()));
    *canceled = true;
}

// S2: draw the Balanced Hardcore button in the Game tab of the Create New World screen.
pub(super) fn on_screen_render_post(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use horizon_client::text::Text;
    track_screen(game);
    let Some(rect) = button_rect(game, screen) else {
        return;
    };
    let locked = create_locked(game);
    let hovered = !locked && logic::inside(rect, screen.mouse);
    let sprite = if locked {
        "gui/sprites/widget/button_disabled"
    } else if hovered {
        "gui/sprites/widget/button_highlighted"
    } else {
        "gui/sprites/widget/button"
    };
    // The fill shows if the sprite is missing; the sprite covers it otherwise.
    game.native_fill(batch, rect, 0xFF30_3030);
    game.native_gui_nine_slice(batch, sprite, rect, 0xFFFF_FFFF);
    let state = if create_choice() { "options.on" } else { "options.off" };
    let label = Text::option_name_value(
        Text::translate("bmh.createWorld.balancedHardcore", &[]),
        Text::translate(state, &[]),
    )
    .plain();
    let width = game.native_text_width(&label);
    let x = rect[0] + ((rect[2] - width) / 2.0).floor();
    let y = rect[1] + (rect[3] - 8.0) / 2.0;
    let color = if locked { 0xA0_A0A0 } else { 0xFF_FFFF };
    game.native_text(batch, &label, [x, y], color, !locked);
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

/// The path of the config file.
fn config_path() -> Result<PathBuf, String> {
    crate::game::Game::native_config_dir()
        .map(|dir| dir.join(logic::CONFIG_FILE))
        .map_err(|error| error.to_string())
}

/// Takes the values of the config text the game read. A value outside its range gets the default of
/// its key, and the file is written again with the corrected values, as NeoForge's `ModConfigSpec`
/// does. The write happens once: the corrected file asks for no second write, and a second hook
/// call that still holds the old text finds the file already correct and leaves it alone.
fn apply_config(config: &crate::game::native_configs::NativeConfigFile) {
    if config.mod_id != crate::game::bmh::ID
        || config.kind != crate::game::native_configs::NativeConfigKind::Common
    {
        return;
    }
    let (values, corrected) = logic::parse_config_checked(&config.text);
    *lock(&COMMON_CONFIG) = values;
    if corrected {
        write_corrected_config(values);
    }
}

/// Writes the corrected config file unless the file already holds that text.
fn write_corrected_config(values: Config) {
    let text = logic::config_text(values);
    let written = config_path().and_then(|path| {
        if std::fs::read_to_string(&path).is_ok_and(|current| current == text) {
            return Ok(false);
        }
        std::fs::write(&path, &text).map(|()| true).map_err(|error| error.to_string())
    });
    match written {
        Ok(true) => log::info!(
            "[BalancedMcHardcoreMain] The config file held a value outside its range. It is written again with the default of that key."
        ),
        Ok(false) => {}
        Err(error) => log::warn!("[BalancedMcHardcoreMain] The corrected config file is not saved: {error}"),
    }
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
    buttons.push(Text::translate("bmh.config.edit", &[]).plain());
}

// S4: a click on a stepper button of the BMH page. The new values go to the config file at once and
// to the cache. The game reads the file again within a second and gives the same values back.
pub(super) fn on_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    if mod_id != crate::game::bmh::ID {
        return;
    }
    let Some((field, raise)) = logic::button(button) else {
        if button == logic::buttons().len() {
            crate::game::dialogs::bmh_config_editor::open(game, *lock(&COMMON_CONFIG));
        }
        return;
    };
    let current = *lock(&COMMON_CONFIG);
    let next = logic::step(current, field, raise);
    if next == current {
        return;
    }
    if let Err(error) = save_config(next) {
        log::warn!("[BalancedMcHardcoreMain] The config file is not saved: {error}");
    }
}

/// Apply the values after the local config file accepts them.
pub(in crate::game) fn save_config(next: Config) -> Result<(), String> {
    let path = config_path()?;
    std::fs::write(path, logic::config_text(next)).map_err(|error| error.to_string())?;
    *lock(&COMMON_CONFIG) = next;
    Ok(())
}
