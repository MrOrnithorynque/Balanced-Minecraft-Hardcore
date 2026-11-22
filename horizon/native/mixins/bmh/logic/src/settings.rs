// Area settings (owner: the settings agent).
// It holds the six world rules (their file text, the import of a Java world, the first value of a
// world, the `/bmh rule` command), the config file `config/bmh-common.toml` (text, parse, the
// stepper buttons of the Options > Mods page) and the Create New World button (its choice and its
// place). No game type here.
// Java reference: utilities/BMHGameRules, utilities/BMHConfigLimits, neoforge/NeoForgeConfig,
// client/config/ConfigScreen, mixin/CreateWorldGameTabMixin, mixin/N_SavedDataGameRulesMixin.

use super::contract::{Config, Rule};

// ---------------------------------------------------------------------------------------------
// The six world rules (S1)
// ---------------------------------------------------------------------------------------------

/// The values of the six rules of one world (BMHGameRules.java:27-37). The index of a rule is its
/// place in `Rule::ALL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules([bool; 6]);

impl Rules {
    /// A new world: every rule on (BMHGameRules.java:27-37, `registerBoolean(name, true)`).
    pub const DEFAULT: Rules = Rules([true; 6]);

    /// Every rule set to `value`: what the Create New World button does
    /// (CreateWorldGameTabMixin.java:62-71, `setBalancedHardcoreEnabled`).
    pub fn all(value: bool) -> Rules {
        Rules([value; 6])
    }

    /// The value of one rule.
    pub fn get(self, rule: Rule) -> bool {
        self.0[index(rule)]
    }

    /// The same rules with one value changed.
    pub fn with(self, rule: Rule, value: bool) -> Rules {
        let mut values = self.0;
        values[index(rule)] = value;
        Rules(values)
    }

    /// True when any rule is on (CreateWorldGameTabMixin.java:57-60, `isBalancedHardcoreEnabled`).
    pub fn any(self) -> bool {
        self.0.iter().any(|value| *value)
    }
}

/// The place of a rule in `Rule::ALL` (BMHGameRules.java:40, `ALL_RULES`).
fn index(rule: Rule) -> usize {
    match rule {
        Rule::RandomDeathSpawnPoint => 0,
        Rule::DisplayTextOnRespawn => 1,
        Rule::FoodAndHealthRegenOnWakeUp => 2,
        Rule::OnlyCarryOneEternalItemType => 3,
        Rule::DamageEternalItemWhenDead => 4,
        Rule::RemoveCoordinatesInSurvival => 5,
    }
}

/// The folder of the mod inside the world folder. The world's backups carry it.
pub const WORLD_FOLDER: &str = "bmh";

/// The file of the six rules inside `WORLD_FOLDER`.
pub const RULES_FILE: &str = "rules.txt";

/// The text of the rules file: one line `<name> = true|false` for each rule.
pub fn rules_text(rules: Rules) -> String {
    let mut text = String::from(
        "# Balanced Minecraft Hardcore: the world rules of this world.\n\
         # Change one in the game with: /bmh rule <name> true|false\n",
    );
    for rule in Rule::ALL {
        text.push_str(rule.name());
        text.push_str(" = ");
        text.push_str(bool_text(rules.get(rule)));
        text.push('\n');
    }
    text
}

/// The rules in a rules file. A line that is not understood is skipped. A rule that the file does
/// not name stays on, as a rule that is missing from the Java saved data does.
pub fn parse_rules(text: &str) -> Rules {
    read_rule_entries(text).unwrap_or(Rules::DEFAULT)
}

fn read_rule_entries(text: &str) -> Option<Rules> {
    let mut rules = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        let (Some(rule), Some(value)) = (Rule::from_name(name.trim()), parse_bool(value.trim())) else {
            continue;
        };
        rules = Some(rules.unwrap_or(Rules::DEFAULT).with(rule, value));
    }
    rules
}

/// `true` or `false`, in lower case as Brigadier's bool argument reads them.
pub fn parse_bool(text: &str) -> Option<bool> {
    match text {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// The text of a value.
pub fn bool_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// A value of the Java game rules saved data, as the module reads it from the NBT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Saved {
    /// A string tag. Before Minecraft 26.1 a game rule was saved as text.
    Text(String),
    /// A byte tag. Since 26.1 a boolean game rule is saved as a byte.
    Byte(i8),
    /// Any other tag.
    Other,
}

/// The value of a saved rule (BMHGameRules.java:67-69): text is read as `Boolean.parseBoolean`
/// does, a byte is true when it is not 0, any other tag counts as true (`orElse(true)`).
pub fn saved_value(saved: &Saved) -> bool {
    match saved {
        Saved::Text(text) => text.eq_ignore_ascii_case("true"),
        Saved::Byte(byte) => *byte != 0,
        Saved::Other => true,
    }
}

/// The keys under which the Java game rules saved data can hold a rule, in the order to read them:
/// the 26.1 id first, then the name from before 26.1. The old name comes last because the Java
/// migration renames it to the id and so overwrites the id (BMHGameRules.java:60-72).
pub fn saved_keys(rule: Rule) -> [String; 2] {
    [format!("bmh:{}", rule.name()), rule.legacy_name().to_owned()]
}

/// The rules of a Java world, read from its game rules saved data. `read` answers the saved value
/// of a key. None when the file holds no BMH rule at all. A rule that is missing stays on, as it
/// does in Java.
pub fn import_rules(read: impl Fn(&str) -> Option<Saved>) -> Option<Rules> {
    let mut rules = Rules::DEFAULT;
    let mut found = false;
    for rule in Rule::ALL {
        for key in saved_keys(rule) {
            if let Some(saved) = read(&key) {
                rules = rules.with(rule, saved_value(&saved));
                found = true;
            }
        }
    }
    found.then_some(rules)
}

/// The rules of a world the first time the mod opens it. In this order:
/// 1. the rules file of the world, when there is one;
/// 2. the values of a Java world's saved data;
/// 3. for a new world, the Create New World choice (GAPS S2);
/// 4. every rule on.
pub fn initial_rules(
    file: Option<Rules>,
    imported: Option<Rules>,
    new_world: bool,
    choice: Option<Rules>,
) -> Rules {
    if let Some(rules) = file {
        return rules;
    }
    if let Some(rules) = imported {
        return rules;
    }
    match (new_world, choice) {
        (true, Some(rules)) => rules,
        _ => Rules::DEFAULT,
    }
}

// ---------------------------------------------------------------------------------------------
// The command /bmh rule (S1)
// ---------------------------------------------------------------------------------------------

/// The level an operator needs: the one of vanilla's `/gamerule` (`LEVEL_GAMEMASTERS`).
pub const COMMAND_LEVEL: u8 = 2;

/// True when a player of operator level `permission` may run `/bmh rule`. Horizon checks one level
/// for the whole `/bmh` root: the lowest level of all its usages. `/bmh point` is level 0, as in Java,
/// so every player reaches this area's hook and the area refuses by itself.
pub fn may_run_rule(permission: u8) -> bool {
    permission >= COMMAND_LEVEL
}

/// The usages of `/bmh rule`, as the registration of a command reads them. The amulet area adds
/// its own `bmh point` usages: the server merges the usages that share the word `bmh`.
pub const COMMAND_USAGES: [&str; 3] = [
    "bmh rule",
    "bmh rule <name:word?>",
    "bmh rule <name:word?> <value:boolean>",
];

/// The namespace of the BMH rules in `/gamerule` (the id of `BMHGameRules`: `bmh:<name>`).
pub const GAMERULE_NAMESPACE: &str = "bmh:";

/// A typed `/gamerule bmh:<name> [value]` as the line `bmh rule <name> [value]` that the settings
/// area runs. None when the line is not a `gamerule` line for a rule of the `bmh` namespace. The
/// vanilla rules (`gamerule keep_inventory true`) stay vanilla's. The namespace test is exact:
/// `bmh:` and the name after it; the name is not checked here, `parse_rule_command` does that.
pub fn gamerule_line_to_bmh(line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    if words.next()? != "gamerule" {
        return None;
    }
    let name = words.next()?.strip_prefix(GAMERULE_NAMESPACE)?;
    let rest: Vec<&str> = words.collect();
    let line = format!("bmh rule {name} {}", rest.join(" "));
    Some(line.trim_end().to_owned())
}

/// The rule names that begin with `typed`, for the completion of `bmh rule <name>`. A typed `bmh:`
/// prefix is allowed, as `/gamerule` takes the id.
pub fn rule_name_matches(typed: &str) -> Vec<&'static str> {
    let typed = typed.strip_prefix(GAMERULE_NAMESPACE).unwrap_or(typed);
    Rule::ALL.iter().map(|rule| rule.name()).filter(|name| name.starts_with(typed)).collect()
}

/// Vanilla's text of `/gamerule <rule>`: "Game rule %s is currently set to: %s".
pub const KEY_QUERY: &str = "commands.gamerule.query";
/// Vanilla's text of `/gamerule <rule> <value>`: "Game rule %s has been updated to: %s".
pub const KEY_SET: &str = "commands.gamerule.set";
/// "Unknown BMH rule: %s" (the rule name that was typed).
pub const KEY_UNKNOWN: &str = "bmh.rule.unknown";
/// "Expected true or false, found: %s" (the word that was typed).
pub const KEY_BAD_VALUE: &str = "bmh.rule.bad_value";
/// "Usage: /bmh rule [<rule> [true|false]]".
pub const KEY_USAGE: &str = "bmh.rule.usage";

/// What a command line asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleCommand {
    /// The line is not `bmh rule ...`: another area's command.
    NotMine,
    /// `bmh rule`: show every rule.
    List,
    /// `bmh rule <name>`: show one rule.
    Query(Rule),
    /// `bmh rule <name> <value>`: change one rule.
    Set(Rule, bool),
    /// The name is no BMH rule.
    UnknownRule(String),
    /// The value is neither `true` nor `false`.
    BadValue(String),
    /// Too many words.
    Usage,
}

/// Reads a command line (without its slash).
pub fn parse_rule_command(line: &str) -> RuleCommand {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.first() != Some(&"bmh") || words.get(1) != Some(&"rule") {
        return RuleCommand::NotMine;
    }
    let (Some(name), value, rest) = (words.get(2), words.get(3), words.get(4)) else {
        return RuleCommand::List;
    };
    if rest.is_some() {
        return RuleCommand::Usage;
    }
    let Some(rule) = Rule::from_name(name) else {
        return RuleCommand::UnknownRule((*name).to_owned());
    };
    match value {
        None => RuleCommand::Query(rule),
        Some(word) => match parse_bool(word) {
            Some(value) => RuleCommand::Set(rule, value),
            None => RuleCommand::BadValue((*word).to_owned()),
        },
    }
}

/// The id of a rule as `/gamerule` shows it: `bmh:random_death_spawn_point`.
pub fn rule_id(rule: Rule) -> String {
    format!("bmh:{}", rule.name())
}

// ---------------------------------------------------------------------------------------------
// The config file (S3)
// ---------------------------------------------------------------------------------------------

/// The key of the shortest distance (NeoForgeConfig.java:23).
pub const KEY_MIN_DISTANCE: &str = "minDistanceFromBed";
/// The key of the longest distance (NeoForgeConfig.java:28).
pub const KEY_MAX_DISTANCE: &str = "maxDistanceFromBed";
/// The key of the most amulet points (NeoForgeConfig.java:33).
pub const KEY_MAX_POINTS: &str = "maxAmuletPoints";

/// The file name inside `config/`: `<mod id>-common.toml`, as the NeoForge build writes it.
pub const CONFIG_FILE: &str = "bmh-common.toml";

/// The text of a config file, in the layout NeoForge's `ModConfigSpec` writes: the Java comment of
/// each key, its range, then the value (NeoForgeConfig.java:22-35).
pub fn config_text(config: Config) -> String {
    format!(
        "#Minimum distance from bed to respawn\n\
         #Range: {min} ~ {max}\n\
         {KEY_MIN_DISTANCE} = {}\n\
         #Maximum distance from bed to respawn\n\
         #Range: {min} ~ {max}\n\
         {KEY_MAX_DISTANCE} = {}\n\
         #Most points a player can keep on the Navigator's Amulet\n\
         #Range: {pmin} ~ {pmax}\n\
         {KEY_MAX_POINTS} = {}\n",
        config.min_distance,
        config.max_distance,
        config.max_amulet_points,
        min = Config::MIN_DISTANCE,
        max = Config::MAX_DISTANCE,
        pmin = Config::MIN_AMULET_POINTS,
        pmax = Config::MAX_AMULET_POINTS,
    )
}

/// The values of a config file. A key that is missing, is no integer or is outside its range gets
/// its default: `ModConfigSpec` corrects such a value to the default of the key.
pub fn parse_config(text: &str) -> Config {
    parse_config_checked(text).0
}

/// The values of a config file, and whether the file holds a value that was corrected: a known key
/// whose value is no integer or is outside its range. A missing key is not a correction, and nor is
/// a minimum above the maximum (the Java mod keeps it too). NeoForge's `ModConfigSpec` writes the
/// corrected file back, so the module does the same once when this answers true.
pub fn parse_config_checked(text: &str) -> (Config, bool) {
    let mut config = Config::DEFAULT;
    let mut corrected = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if ![KEY_MIN_DISTANCE, KEY_MAX_DISTANCE, KEY_MAX_POINTS].contains(&key) {
            continue;
        }
        // A comment may follow the value on the same line.
        let value = value.split('#').next().unwrap_or("").trim();
        let Some(value) = value.parse::<i32>().ok().filter(|value| in_range(key, *value)) else {
            corrected = true;
            continue;
        };
        match key {
            KEY_MIN_DISTANCE => config.min_distance = value,
            KEY_MAX_DISTANCE => config.max_distance = value,
            _ => config.max_amulet_points = value,
        }
    }
    (config, corrected)
}

/// True when `value` is inside the range of the config key `key`.
fn in_range(key: &str, value: i32) -> bool {
    if key == KEY_MAX_POINTS {
        (Config::MIN_AMULET_POINTS..=Config::MAX_AMULET_POINTS).contains(&value)
    } else {
        (Config::MIN_DISTANCE..=Config::MAX_DISTANCE).contains(&value)
    }
}

// ---------------------------------------------------------------------------------------------
// The config page, Options > Mods (S4)
// ---------------------------------------------------------------------------------------------

/// One value of the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    MinDistance,
    MaxDistance,
    AmuletPoints,
}

/// A distance moves by this many blocks for each click. Both Java defaults (5000 and 60000) and
/// the lowest value are multiples of it.
pub const DISTANCE_STEP: i32 = 5000;

/// The amulet points move by this many for each click.
pub const POINTS_STEP: i32 = 1;

/// The buttons of the page, in order: for each value, one that raises it and one that lowers it.
const BUTTONS: [(Field, bool); 6] = [
    (Field::MinDistance, true),
    (Field::MinDistance, false),
    (Field::MaxDistance, true),
    (Field::MaxDistance, false),
    (Field::AmuletPoints, true),
    (Field::AmuletPoints, false),
];

/// What button `index` does: the value and the direction (true raises).
pub fn button(index: usize) -> Option<(Field, bool)> {
    BUTTONS.get(index).copied()
}

/// All the buttons, to build the labels.
pub fn buttons() -> &'static [(Field, bool)] {
    &BUTTONS
}

/// The value of a field.
pub fn field_value(config: Config, field: Field) -> i32 {
    match field {
        Field::MinDistance => config.min_distance,
        Field::MaxDistance => config.max_distance,
        Field::AmuletPoints => config.max_amulet_points,
    }
}

/// The sign on a button.
pub fn sign(raise: bool) -> &'static str {
    if raise { "+" } else { "-" }
}

/// The lang key of the short name of a field (the Java keys carry the range in their text).
pub fn field_key(field: Field) -> &'static str {
    match field {
        Field::MinDistance => "bmh.config.short.minDistanceFromBed",
        Field::MaxDistance => "bmh.config.short.maxDistanceFromBed",
        Field::AmuletPoints => "bmh.config.short.maxAmuletPoints",
    }
}

/// The lang key of a button label: "[%s] %s: %s" (sign, name, value).
pub const KEY_STEP: &str = "bmh.config.step";

/// The config after one click. The value stays in the Java range
/// (BMHConfigLimits.java:9-16). The minimum distance never goes above the maximum, which is the
/// rule of the Java screen (ConfigScreen.java:104-106, `min > max` refuses Done). A click that
/// would break a rule changes nothing.
pub fn step(config: Config, field: Field, raise: bool) -> Config {
    let sign = if raise { 1 } else { -1 };
    let mut next = config;
    match field {
        Field::MinDistance => {
            next.min_distance = (config.min_distance + sign * DISTANCE_STEP)
                .clamp(Config::MIN_DISTANCE, Config::MAX_DISTANCE);
        }
        Field::MaxDistance => {
            next.max_distance = (config.max_distance + sign * DISTANCE_STEP)
                .clamp(Config::MIN_DISTANCE, Config::MAX_DISTANCE);
        }
        Field::AmuletPoints => {
            next.max_amulet_points = (config.max_amulet_points + sign * POINTS_STEP)
                .clamp(Config::MIN_AMULET_POINTS, Config::MAX_AMULET_POINTS);
        }
    }
    if next.min_distance > next.max_distance && next.min_distance > config.min_distance {
        return config;
    }
    if next.min_distance > next.max_distance && next.max_distance < config.max_distance {
        return config;
    }
    next
}

// ---------------------------------------------------------------------------------------------
// The Create New World button (S2)
// ---------------------------------------------------------------------------------------------

/// The id of the screen that gets the button (`NativeScreenView.id`).
pub const CREATE_SCREEN: &str = "world_create";

/// The ids of the screens that belong to one Create New World session: the screen itself and the two
/// editors it opens from the World tab (`NativeScreenView.id`, native-mods.md API 326). The choice of
/// the button stays while the player moves between them.
pub fn is_create_screen(id: &str) -> bool {
    matches!(id, "world_create" | "world_create_flat" | "world_create_buffet")
}

/// The name of the mod setting that keeps the choice, in the mod's data folder. It is the channel
/// to the server, which reads it when it opens the new world. Each new Create New World screen
/// stores its rules before the player can create the world.
pub const CHOICE_SETTING: &str = "balanced_hardcore";

/// The width of the button in GUI pixels. The Java button is 210 wide (CreateWorldGameTabMixin.java:41),
/// as wide as the buttons of the Game tab.
pub const BUTTON_WIDTH: f32 = 210.0;

/// The height of the button (CreateWorldGameTabMixin.java:41).
pub const BUTTON_HEIGHT: f32 = 20.0;

/// The index of the Game tab in the tab bar (Game 0, World 1, More 2).
pub const GAME_TAB: usize = 0;

// The layout of the Game tab in Horizon's title screen (Nightly fdc87aec, `title/create_world.rs`,
// `grid`): the tab bar is 24 high, the footer 33, and the tab's grid is 210 wide and 117 high. It
// sits at half of the free width and one sixth of the free height. Name box, Game Mode, Difficulty
// and Allow Commands are in the grid; Allow Commands ends at its bottom edge.
const TAB_BAR: f32 = 24.0;
const FOOTER: f32 = 33.0;
const GAME_GRID_HEIGHT: f32 = 117.0;
/// The gap between two rows of the tab (a 20 high button and 8 pixels, `gy + 41`, `+ 69`, `+ 97`).
const ROW_GAP: f32 = 8.0;

/// The button of the Game tab: 210 by 20, in the next row under Allow Commands, as the Java button
/// is (CreateWorldGameTabMixin.java:50-52, one more row of the tab's grid). `window` is `[x, y,
/// width, height]` of the screen in GUI pixels and `tab` the open tab. None when the open tab is
/// not the Game tab, and when the window is so low that the row would reach the footer's buttons.
/// The arithmetic is the game's own (`ceil` of the size, then `trunc`), so the button lines up
/// with the buttons above it.
pub fn button_rect(window: [f32; 4], tab: usize) -> Option<[f32; 4]> {
    if tab != GAME_TAB {
        return None;
    }
    let [x, y, width, height] = window;
    let (width, height) = (width.ceil(), height.ceil());
    let area = height - TAB_BAR - FOOTER;
    let grid_x = ((width - BUTTON_WIDTH) * 0.5).trunc();
    let grid_y = TAB_BAR + ((area - GAME_GRID_HEIGHT) * 0.166_666_67).trunc();
    let top = grid_y + GAME_GRID_HEIGHT + ROW_GAP;
    (top + BUTTON_HEIGHT <= height - FOOTER).then_some([x + grid_x, y + top, BUTTON_WIDTH, BUTTON_HEIGHT])
}

/// True when the pointer is inside the rectangle.
pub fn inside(rect: [f32; 4], pointer: [f32; 2]) -> bool {
    let [x, y, width, height] = rect;
    pointer[0] >= x && pointer[0] < x + width && pointer[1] >= y && pointer[1] < y + height
}

/// Keep the original boolean format for uniform rules. Mixed worlds use the world rules file format.
pub fn choice_text(rules: Rules) -> String {
    let enabled = rules.any();
    if rules == Rules::all(enabled) {
        bool_text(enabled).to_owned()
    } else {
        rules_text(rules)
    }
}

pub fn parse_choice(text: &str) -> Option<Rules> {
    parse_bool(text.trim()).map(Rules::all).or_else(|| read_rule_entries(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gamerule_line_of_the_bmh_namespace_becomes_a_bmh_rule_line() {
        assert_eq!(
            gamerule_line_to_bmh("gamerule bmh:random_death_spawn_point false").as_deref(),
            Some("bmh rule random_death_spawn_point false")
        );
        assert_eq!(
            gamerule_line_to_bmh("gamerule bmh:display_text_on_respawn").as_deref(),
            Some("bmh rule display_text_on_respawn")
        );
        assert_eq!(gamerule_line_to_bmh("gamerule bmh:").as_deref(), Some("bmh rule"));
        // Whatever follows is kept, so that the rule parser answers with its own errors.
        assert_eq!(gamerule_line_to_bmh("gamerule bmh:x maybe now").as_deref(), Some("bmh rule x maybe now"));
    }

    #[test]
    fn a_vanilla_gamerule_line_is_left_to_vanilla() {
        assert_eq!(gamerule_line_to_bmh("gamerule keep_inventory true"), None);
        assert_eq!(gamerule_line_to_bmh("gamerule minecraft:pvp false"), None);
        assert_eq!(gamerule_line_to_bmh("gamerule"), None);
        assert_eq!(gamerule_line_to_bmh("gamerule random_death_spawn_point false"), None);
        assert_eq!(gamerule_line_to_bmh("bmh rule x"), None);
        assert_eq!(gamerule_line_to_bmh("gamerulebmh:x"), None);
        assert_eq!(gamerule_line_to_bmh("tell bmh:x"), None);
    }

    #[test]
    fn the_converted_line_is_read_by_the_rule_parser() {
        let line = gamerule_line_to_bmh("gamerule bmh:damage_eternal_item_when_dead false").unwrap();
        assert_eq!(
            parse_rule_command(&line),
            RuleCommand::Set(Rule::from_name("damage_eternal_item_when_dead").unwrap(), false)
        );
        let line = gamerule_line_to_bmh("gamerule bmh:nonsense").unwrap();
        assert_eq!(parse_rule_command(&line), RuleCommand::UnknownRule("nonsense".to_owned()));
    }

    #[test]
    fn the_button_of_a_recreated_world_is_on_when_any_rule_is_on() {
        assert!(Rules::DEFAULT.any());
        assert!(!Rules::all(false).any());
        assert!(Rules::all(false).with(Rule::ALL[3], true).any());
        assert!(parse_rules("random_death_spawn_point = false\n").any());
    }

    #[test]
    fn the_completion_lists_the_rules_that_match() {
        assert_eq!(rule_name_matches("").len(), 6);
        assert_eq!(rule_name_matches("random"), vec!["random_death_spawn_point"]);
        assert_eq!(rule_name_matches("bmh:display"), vec!["display_text_on_respawn"]);
        assert!(rule_name_matches("zzz").is_empty());
    }

    #[test]
    fn rule_command_needs_the_gamerule_level() {
        assert!(!may_run_rule(0));
        assert!(!may_run_rule(1));
        assert!(may_run_rule(2));
        assert!(may_run_rule(4));
    }

    #[test]
    fn rule_command_asks_the_level_only_for_its_own_lines() {
        // The hook refuses a level 0 player only after parse_rule_command knows the line.
        assert_eq!(parse_rule_command("bmh point save 1 2 3 Home"), RuleCommand::NotMine);
        assert_ne!(parse_rule_command("bmh rule"), RuleCommand::NotMine);
    }

    #[test]
    fn rules_start_on_and_flip_together() {
        for rule in Rule::ALL {
            assert!(Rules::DEFAULT.get(rule));
            assert!(!Rules::all(false).get(rule));
        }
        assert!(Rules::DEFAULT.any());
        assert!(!Rules::all(false).any());
        // One rule on is enough for the button to read ON (CreateWorldGameTabMixin.java:57-60).
        assert!(Rules::all(false).with(Rule::DamageEternalItemWhenDead, true).any());
    }

    #[test]
    fn one_rule_changes_alone() {
        let rules = Rules::DEFAULT.with(Rule::DisplayTextOnRespawn, false);
        for rule in Rule::ALL {
            assert_eq!(rules.get(rule), rule != Rule::DisplayTextOnRespawn, "{}", rule.name());
        }
    }

    #[test]
    fn the_rules_file_round_trips() {
        let rules = Rules::DEFAULT
            .with(Rule::RandomDeathSpawnPoint, false)
            .with(Rule::RemoveCoordinatesInSurvival, false);
        assert_eq!(parse_rules(&rules_text(rules)), rules);
        assert_eq!(parse_rules(&rules_text(Rules::all(false))), Rules::all(false));
        assert_eq!(parse_rules(&rules_text(Rules::DEFAULT)), Rules::DEFAULT);
    }

    #[test]
    fn a_damaged_rules_file_keeps_the_rules_on() {
        assert_eq!(parse_rules(""), Rules::DEFAULT);
        assert_eq!(parse_rules("garbage\n= true\nrandom_death_spawn_point = maybe"), Rules::DEFAULT);
        let text = "# comment\n  display_text_on_respawn=false  \nunknown_rule = false\n";
        assert_eq!(parse_rules(text), Rules::DEFAULT.with(Rule::DisplayTextOnRespawn, false));
    }

    #[test]
    fn a_saved_value_is_read_as_java_reads_it() {
        // BMHGameRules.java:67-69.
        assert!(saved_value(&Saved::Text("true".into())));
        assert!(saved_value(&Saved::Text("TRUE".into())));
        assert!(!saved_value(&Saved::Text("false".into())));
        assert!(!saved_value(&Saved::Text("yes".into())));
        assert!(saved_value(&Saved::Byte(1)));
        assert!(!saved_value(&Saved::Byte(0)));
        assert!(saved_value(&Saved::Other));
    }

    #[test]
    fn a_java_world_is_imported_from_the_new_ids() {
        let read = |key: &str| match key {
            "bmh:random_death_spawn_point" => Some(Saved::Byte(0)),
            "bmh:only_carry_one_eternal_item_type" => Some(Saved::Byte(1)),
            "minecraft:keep_inventory" => Some(Saved::Byte(0)),
            _ => None,
        };
        let rules = import_rules(read);
        assert_eq!(rules, Some(Rules::DEFAULT.with(Rule::RandomDeathSpawnPoint, false)));
    }

    #[test]
    fn a_world_from_before_26_1_is_imported_from_the_old_names() {
        let read = |key: &str| match key {
            "displayTextOnRespawn" => Some(Saved::Text("false".into())),
            "foodAndHealthRegenOnWakeUp" => Some(Saved::Text("true".into())),
            _ => None,
        };
        let rules = import_rules(read);
        assert_eq!(rules, Some(Rules::DEFAULT.with(Rule::DisplayTextOnRespawn, false)));
    }

    #[test]
    fn the_old_name_wins_as_the_java_rename_does() {
        // Both keys in one file: the Java migration writes the old value over the id.
        let read = |key: &str| match key {
            "bmh:damage_eternal_item_when_dead" => Some(Saved::Byte(1)),
            "damageEternalItemWhenDead" => Some(Saved::Text("false".into())),
            _ => None,
        };
        let rules = import_rules(read).map(|rules| rules.get(Rule::DamageEternalItemWhenDead));
        assert_eq!(rules, Some(false));
    }

    #[test]
    fn a_file_without_bmh_rules_imports_nothing() {
        assert_eq!(import_rules(|_| None), None);
        assert_eq!(import_rules(|key| (key == "minecraft:keep_inventory").then_some(Saved::Byte(0))), None);
    }

    #[test]
    fn the_first_value_of_a_world_follows_the_order() {
        let off = Rules::all(false);
        let mixed = Rules::DEFAULT.with(Rule::RandomDeathSpawnPoint, false);
        // The file of the world wins over everything.
        assert_eq!(initial_rules(Some(mixed), Some(off), true, Some(off)), mixed);
        // Then a Java world.
        assert_eq!(initial_rules(None, Some(mixed), true, Some(off)), mixed);
        // Then the choice, for a new world only.
        assert_eq!(initial_rules(None, None, true, Some(off)), off);
        assert_eq!(initial_rules(None, None, true, Some(Rules::DEFAULT)), Rules::DEFAULT);
        assert_eq!(initial_rules(None, None, false, Some(off)), Rules::DEFAULT);
        assert_eq!(initial_rules(None, None, true, None), Rules::DEFAULT);
    }

    #[test]
    fn the_command_is_read_word_by_word() {
        let rule = Rule::OnlyCarryOneEternalItemType;
        assert_eq!(parse_rule_command("bmh rule"), RuleCommand::List);
        assert_eq!(parse_rule_command("bmh  rule   "), RuleCommand::List);
        assert_eq!(
            parse_rule_command("bmh rule only_carry_one_eternal_item_type"),
            RuleCommand::Query(rule)
        );
        assert_eq!(
            parse_rule_command("bmh rule bmh:only_carry_one_eternal_item_type false"),
            RuleCommand::Set(rule, false)
        );
        assert_eq!(
            parse_rule_command("bmh rule onlyCarryOneEternalItemType true"),
            RuleCommand::Set(rule, true)
        );
    }

    #[test]
    fn a_wrong_command_is_refused() {
        assert_eq!(
            parse_rule_command("bmh rule keep_inventory true"),
            RuleCommand::UnknownRule("keep_inventory".into())
        );
        assert_eq!(
            parse_rule_command("bmh rule random_death_spawn_point maybe"),
            RuleCommand::BadValue("maybe".into())
        );
        // Brigadier reads the bool argument in lower case only.
        assert_eq!(
            parse_rule_command("bmh rule random_death_spawn_point True"),
            RuleCommand::BadValue("True".into())
        );
        assert_eq!(parse_rule_command("bmh rule random_death_spawn_point true x"), RuleCommand::Usage);
    }

    #[test]
    fn another_command_is_not_mine() {
        assert_eq!(parse_rule_command("bmh point save 1 2 3 Home"), RuleCommand::NotMine);
        assert_eq!(parse_rule_command("bmh"), RuleCommand::NotMine);
        assert_eq!(parse_rule_command("gamerule keep_inventory"), RuleCommand::NotMine);
        assert_eq!(parse_rule_command(""), RuleCommand::NotMine);
        assert_eq!(rule_id(Rule::DisplayTextOnRespawn), "bmh:display_text_on_respawn");
    }

    #[test]
    fn the_config_text_has_the_neoforge_layout() {
        let text = config_text(Config::DEFAULT);
        let expected = "#Minimum distance from bed to respawn\n\
                        #Range: 5000 ~ 200000\n\
                        minDistanceFromBed = 5000\n\
                        #Maximum distance from bed to respawn\n\
                        #Range: 5000 ~ 200000\n\
                        maxDistanceFromBed = 60000\n\
                        #Most points a player can keep on the Navigator's Amulet\n\
                        #Range: 1 ~ 16\n\
                        maxAmuletPoints = 5\n";
        assert_eq!(text, expected);
    }

    #[test]
    fn the_config_round_trips() {
        let config = Config { min_distance: 10_000, max_distance: 150_000, max_amulet_points: 16 };
        assert_eq!(parse_config(&config_text(config)), config);
        assert_eq!(parse_config(&config_text(Config::DEFAULT)), Config::DEFAULT);
        assert_eq!(parse_config(""), Config::DEFAULT);
    }

    #[test]
    fn a_config_value_outside_its_range_gets_its_default() {
        let text = "minDistanceFromBed = 100\nmaxDistanceFromBed = 9000000\nmaxAmuletPoints = 0\n";
        assert_eq!(parse_config(text), Config::DEFAULT);
        let text = "minDistanceFromBed = 200000\nmaxDistanceFromBed = 5000\nmaxAmuletPoints = 16\n";
        // Both are in range, so min above max stays: the respawn area handles it.
        let config = parse_config(text);
        assert_eq!(config.min_distance, 200_000);
        assert_eq!(config.max_distance, 5000);
        assert_eq!(config.max_amulet_points, 16);
    }

    #[test]
    fn a_damaged_config_line_is_skipped() {
        let text = "minDistanceFromBed = abc\n[section]\nmaxDistanceFromBed = 7000 # a comment\nnoequals\n";
        let config = parse_config(text);
        assert_eq!(config.min_distance, 5000);
        assert_eq!(config.max_distance, 7000);
    }

    #[test]
    fn a_click_moves_a_value_by_one_step() {
        let config = Config::DEFAULT;
        assert_eq!(step(config, Field::MaxDistance, true).max_distance, 65_000);
        assert_eq!(step(config, Field::MaxDistance, false).max_distance, 55_000);
        assert_eq!(step(config, Field::MinDistance, true).min_distance, 10_000);
        assert_eq!(step(config, Field::AmuletPoints, true).max_amulet_points, 6);
        assert_eq!(step(config, Field::AmuletPoints, false).max_amulet_points, 4);
    }

    #[test]
    fn a_click_stops_at_the_java_range() {
        let low = Config { min_distance: 5000, max_distance: 5000, max_amulet_points: 1 };
        assert_eq!(step(low, Field::MinDistance, false), low);
        assert_eq!(step(low, Field::AmuletPoints, false), low);
        let high = Config { min_distance: 200_000, max_distance: 200_000, max_amulet_points: 16 };
        assert_eq!(step(high, Field::MaxDistance, true), high);
        assert_eq!(step(high, Field::AmuletPoints, true), high);
    }

    #[test]
    fn the_minimum_never_goes_above_the_maximum() {
        let config = Config { min_distance: 60_000, max_distance: 60_000, max_amulet_points: 5 };
        assert_eq!(step(config, Field::MinDistance, true), config);
        assert_eq!(step(config, Field::MaxDistance, false), config);
        // The other directions are free.
        assert_eq!(step(config, Field::MinDistance, false).min_distance, 55_000);
        assert_eq!(step(config, Field::MaxDistance, true).max_distance, 65_000);
    }

    #[test]
    fn a_hand_edited_file_with_min_above_max_can_be_repaired() {
        let config = Config { min_distance: 100_000, max_distance: 20_000, max_amulet_points: 5 };
        // Lowering the minimum and raising the maximum are allowed.
        assert_eq!(step(config, Field::MinDistance, false).min_distance, 95_000);
        assert_eq!(step(config, Field::MaxDistance, true).max_distance, 25_000);
        // Raising the minimum and lowering the maximum are not.
        assert_eq!(step(config, Field::MinDistance, true), config);
        assert_eq!(step(config, Field::MaxDistance, false), config);
    }

    #[test]
    fn the_page_has_six_buttons_in_pairs() {
        assert_eq!(buttons().len(), 6);
        assert_eq!(button(0), Some((Field::MinDistance, true)));
        assert_eq!(button(1), Some((Field::MinDistance, false)));
        assert_eq!(button(5), Some((Field::AmuletPoints, false)));
        assert_eq!(button(6), None);
        assert_eq!(sign(true), "+");
        assert_eq!(sign(false), "-");
        assert_eq!(field_value(Config::DEFAULT, Field::MaxDistance), 60_000);
    }

    #[test]
    fn the_button_sits_under_allow_commands_in_the_game_tab() {
        // GUI 427 x 240 (2560 x 1440 at the default scale): grid at (108, 35), Allow Commands at y
        // 132 to 152 (gy + 97), so the new row is at y 160, 8 under it.
        let rect = button_rect([0.0, 0.0, 427.0, 240.0], GAME_TAB).expect("room for the row");
        assert_eq!(rect, [108.0, 160.0, 210.0, 20.0]);
        // As wide as the tab's buttons, and on their column.
        assert_eq!(rect[2], 210.0);
        assert_eq!(rect[0], ((427.0_f32.ceil() - 210.0) * 0.5).trunc());
        assert!(inside(rect, [200.0, 170.0]));
        assert!(inside(rect, [108.0, 160.0]));
        assert!(!inside(rect, [318.0, 170.0]));
        assert!(!inside(rect, [200.0, 180.0]));
        assert!(!inside(rect, [107.0, 170.0]));
        // GUI 854 x 480 (GUI scale 3): the free height is 480 - 57 = 423, the grid starts at
        // 24 + trunc((423 - 117) / 6) = 75, so the row is at 75 + 125 = 200.
        let big = button_rect([0.0, 0.0, 854.0, 480.0], GAME_TAB).expect("room for the row");
        assert_eq!(big, [322.0, 200.0, 210.0, 20.0]);
    }

    #[test]
    fn the_button_is_only_in_the_game_tab() {
        let window = [0.0, 0.0, 427.0, 240.0];
        assert!(button_rect(window, 0).is_some());
        assert_eq!(button_rect(window, 1), None);
        assert_eq!(button_rect(window, 2), None);
        assert_eq!(button_rect(window, 9), None);
    }

    #[test]
    fn the_button_never_reaches_the_footer() {
        // The footer's buttons start 33 pixels above the bottom. Find the lowest window that fits
        // the row and check that the button ends above the footer for every height from there.
        for height in 150..1000 {
            let height = height as f32;
            match button_rect([0.0, 0.0, 427.0, height], GAME_TAB) {
                Some(rect) => assert!(rect[1] + rect[3] <= height - 33.0, "height {height}"),
                None => assert!(height < 260.0, "height {height} has room but no button"),
            }
        }
        // The game keeps the GUI at 320 x 240 or more (the highest GUI scale of a window stops
        // there), and the row fits at that size.
        assert!(button_rect([0.0, 0.0, 320.0, 240.0], GAME_TAB).is_some());
        // Only a window under 240 pixels high has no room: the button is left out, so that it never
        // lies over the footer's Create New World and Cancel.
        assert_eq!(button_rect([0.0, 0.0, 320.0, 180.0], GAME_TAB), None);
    }

    #[test]
    fn the_create_session_has_three_screens() {
        assert!(is_create_screen("world_create"));
        assert!(is_create_screen("world_create_flat"));
        assert!(is_create_screen("world_create_buffet"));
        assert!(!is_create_screen("world_select"));
        assert!(!is_create_screen("title"));
        assert!(!is_create_screen("paused"));
        assert_eq!(CREATE_SCREEN, "world_create");
    }

    #[test]
    fn a_corrected_config_value_is_told() {
        // A value outside the range, and a value that is no integer, are corrections.
        let (config, corrected) = parse_config_checked("minDistanceFromBed = 3000\n");
        assert_eq!(config, Config::DEFAULT);
        assert!(corrected);
        assert!(parse_config_checked("maxDistanceFromBed = 999999").1);
        assert!(parse_config_checked("maxAmuletPoints = 99").1);
        assert!(parse_config_checked("maxAmuletPoints = 0").1);
        assert!(parse_config_checked("minDistanceFromBed = abc").1);
        assert!(parse_config_checked("minDistanceFromBed = 5000.0").1);
        // A good value keeps its place and is no correction.
        let (config, corrected) = parse_config_checked(&config_text(Config::DEFAULT));
        assert_eq!(config, Config::DEFAULT);
        assert!(!corrected);
        let good = Config { min_distance: 10_000, max_distance: 150_000, max_amulet_points: 16 };
        assert_eq!(parse_config_checked(&config_text(good)), (good, false));
        // Not corrections: an empty file, a key that is missing, a line of another key, a comment
        // after the value, and a minimum above the maximum (both in range, as in Java).
        assert!(!parse_config_checked("").1);
        assert!(!parse_config_checked("maxAmuletPoints = 7\n").1);
        assert!(!parse_config_checked("other = 12345678\n[section]\nnoequals\n").1);
        assert!(!parse_config_checked("maxDistanceFromBed = 7000 # a comment").1);
        assert!(!parse_config_checked("minDistanceFromBed = 200000\nmaxDistanceFromBed = 5000\n").1);
    }

    #[test]
    fn the_corrected_text_is_clean() {
        // The text the module writes back parses to the same values and asks for no second write.
        let text = "minDistanceFromBed = 3000\nmaxDistanceFromBed = 999999\nmaxAmuletPoints = 99\n";
        let (config, corrected) = parse_config_checked(text);
        assert!(corrected);
        let written = config_text(config);
        assert_eq!(parse_config_checked(&written), (Config::DEFAULT, false));
        // A good key beside a bad one is kept in the corrected file.
        let (config, _) = parse_config_checked("minDistanceFromBed = 12000\nmaxAmuletPoints = 99\n");
        assert_eq!(config.min_distance, 12_000);
        assert_eq!(config.max_amulet_points, 5);
        assert_eq!(parse_config_checked(&config_text(config)), (config, false));
    }

    #[test]
    fn recreate_preserves_mixed_rules_until_the_player_clicks() {
        let mixed = Rules::all(false).with(Rule::DisplayTextOnRespawn, true);
        assert_eq!(initial_rules(None, None, true, parse_choice(&choice_text(mixed))), mixed);
        let clicked = Rules::all(!mixed.any());
        assert_eq!(choice_text(clicked), "false");
        assert_eq!(choice_text(Rules::all(!clicked.any())), "true");
    }

    #[test]
    fn the_choice_is_read_from_the_setting() {
        assert_eq!(parse_choice("true"), Some(Rules::DEFAULT));
        assert_eq!(parse_choice("false\n"), Some(Rules::all(false)));
        assert_eq!(parse_choice(""), None);
        assert_eq!(parse_choice("on"), None);
    }
}
