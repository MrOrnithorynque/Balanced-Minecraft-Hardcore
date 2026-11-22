use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bmh_logic::contract::{Config as Values, Rule};
use bmh_logic::settings::{self as logic, RuleCommand, Rules};
use mod_api::{
    Command, CommandArgumentValue, Config, ConfigFile, ConfigKind, ConfigPath, Flow, HostSide,
    PlayerCommandEvent, RegisterCommandsEvent, RegisterConfigsEvent, RunModCommandEvent,
    ServerHost, Text,
};

static WORLD_RULES: Mutex<Option<(PathBuf, Rules)>> = Mutex::new(None);
static SERVER_CONFIG: Mutex<Values> = Mutex::new(Values::DEFAULT);

fn rules_path(world: &Path) -> PathBuf {
    world.join(logic::WORLD_FOLDER).join(logic::RULES_FILE)
}

fn save_rules(path: &Path, rules: Rules) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, logic::rules_text(rules))?;
    fs::rename(temporary, path)
}

pub(super) fn load_world(host: &dyn ServerHost) {
    let Some(world) = host.world_dir() else {
        return;
    };
    let path = rules_path(&world);
    let rules = match fs::read_to_string(&path) {
        Ok(text) => logic::parse_rules(&text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if let Err(error) = save_rules(&path, Rules::DEFAULT) {
                eprintln!("[BMH] Could not create the world rules: {error}");
            }
            Rules::DEFAULT
        }
        Err(error) => {
            eprintln!("[BMH] Could not read the world rules: {error}");
            Rules::DEFAULT
        }
    };
    *WORLD_RULES
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some((path, rules));
}

fn command_result(host: &dyn ServerHost, command: RuleCommand) -> Vec<Text> {
    let path = host.world_dir().map(|world| rules_path(&world));
    let mut world = WORLD_RULES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let current = world
        .as_mut()
        .filter(|(current, _)| Some(current.as_path()) == path.as_deref());
    let rules = current.as_ref().map_or(Rules::DEFAULT, |(_, rules)| *rules);
    let rule_text = |key, rule, value| {
        Text::translate(
            key,
            vec![
                Text::literal(logic::rule_id(rule)),
                Text::literal(logic::bool_text(value)),
            ],
        )
    };
    match command {
        RuleCommand::List => Rule::ALL
            .iter()
            .map(|&rule| rule_text(logic::KEY_QUERY, rule, rules.get(rule)))
            .collect(),
        RuleCommand::Query(rule) => vec![rule_text(logic::KEY_QUERY, rule, rules.get(rule))],
        RuleCommand::Set(rule, value) => {
            let Some((path, rules)) = current else {
                return vec![Text::literal("The BMH world rules are unavailable.")];
            };
            let updated = rules.with(rule, value);
            if let Err(error) = save_rules(path, updated) {
                eprintln!("[BMH] Could not save the world rules: {error}");
                return vec![Text::literal("The BMH world rules could not be saved.")];
            }
            *rules = updated;
            vec![rule_text(logic::KEY_SET, rule, value)]
        }
        RuleCommand::UnknownRule(name) => vec![Text::translate(
            logic::KEY_UNKNOWN,
            vec![Text::literal(name)],
        )],
        RuleCommand::BadValue(value) => vec![Text::translate(
            logic::KEY_BAD_VALUE,
            vec![Text::literal(value)],
        )],
        RuleCommand::Usage => vec![Text::translate(logic::KEY_USAGE, vec![])],
        RuleCommand::NotMine => Vec::new(),
    }
}

pub(super) fn register_commands(event: &mut RegisterCommandsEvent<'_>) {
    event.commands.push(Command::op(
        "bmh rule [name:word] [value:boolean]",
        logic::COMMAND_LEVEL,
    ));
}

pub(super) fn run_command(host: &mut dyn ServerHost, event: &RunModCommandEvent<'_>) -> Flow {
    if !event
        .command
        .literals
        .iter()
        .map(String::as_str)
        .eq(["bmh", "rule"])
    {
        return Flow::Continue;
    }
    let mut line = String::from("bmh rule");
    if let Some(CommandArgumentValue::String(name)) = event.command.argument("name") {
        line.push(' ');
        line.push_str(name);
    }
    if let Some(CommandArgumentValue::Boolean(value)) = event.command.argument("value") {
        line.push(' ');
        line.push_str(logic::bool_text(*value));
    }
    for reply in command_result(host, logic::parse_rule_command(&line)) {
        let _ = host.commands().feedback(reply);
    }
    Flow::Cancel
}

pub(super) fn rule_alias(host: &dyn ServerHost, event: &mut PlayerCommandEvent<'_>) -> Flow {
    let Some(line) = logic::gamerule_line_to_bmh(event.command) else {
        return Flow::Continue;
    };
    let allowed = host
        .player(event.player)
        .and_then(|player| player.permission_level(host))
        .is_some_and(logic::may_run_rule);
    if allowed {
        event
            .replies
            .extend(command_result(host, logic::parse_rule_command(&line)));
    } else {
        let mut denial = Text::translate("command.unknown.command", vec![]);
        denial.style.color = Some(String::from("red"));
        event.replies.push(denial);
    }
    Flow::Cancel
}

pub(super) fn register_configs(event: &mut RegisterConfigsEvent<'_>) {
    event.configs.push(Config::new(
        crate::ID,
        ConfigKind::Common,
        &logic::config_text(Values::DEFAULT),
    ));
}

pub(super) fn apply_config(host: HostSide, config: &ConfigFile) {
    if config.mod_id != crate::ID || config.kind != ConfigKind::Common {
        return;
    }
    let HostSide::Server(host) = host else { return };
    let (values, corrected) = logic::parse_config_checked(&config.text);
    *SERVER_CONFIG
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = values;
    if corrected {
        let text = logic::config_text(values);
        if let Err(error) = ConfigPath::new(&config.file)
            .and_then(|path| host.config_files().write(&path, text.as_bytes()))
        {
            eprintln!("[BMH] Could not correct the common configuration: {error:?}");
        }
    }
}
