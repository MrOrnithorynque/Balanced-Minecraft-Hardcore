// Values that one area hands to another area on the game side (integrator).
// The two areas and the hook helpers use these types, so no area edits the
// file of another area. docs/horizon/HOOKS.md, "Contracts", says who gives
// and who reads each one.

/// The exile of one distant respawn (Java `RespawnPositionHandler.Exile`,
/// RespawnPositionHandler.java:55). The respawn area gives it once per
/// distant respawn (Java `takeExile`). The homecoming area keeps it in the
/// record (Java `HomecomingHandler.onExile`, HomecomingHandler.java:98).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exile {
    /// The dimension of the player after the respawn, for example
    /// `minecraft:overworld`.
    pub dimension: String,
    /// The block that the exile distance was measured from.
    pub center: [i32; 3],
    /// The distance of the exile in blocks.
    pub distance: i32,
}

/// One named point of the Navigator's Amulet (Java `AmuletPoints.Point`,
/// AmuletPoints.java:42). The amulet area gives the points of a player with
/// `bmh_amulet::points(game, player)`, on the server. The homecoming area
/// calls it in `on_exile`, inside `player_respawned`, and copies the points
/// into the new record when the exile is in the Overworld
/// (HomecomingHandler.java:104-108).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmuletPoint {
    /// The name that the player gave to the point.
    pub name: String,
    /// The bed or lodestone block of the point.
    pub pos: [i32; 3],
}

/// One star reading of the Navigator's Amulet (Java `AmuletReading.record`,
/// AmuletReading.java:210-245). The amulet area writes it on the stack (the
/// compound `bmh_reading` of `minecraft:custom_data`, the Java layout), and in
/// the same use sends it to the player's client in
/// `wire::SkySignal::Reading` on `wire::SKY_CHANNEL`. The sky area reads it
/// in `on_net_from_server` and draws one star chain per azimuth (Java
/// `StarReadingRenderer.startFreshReading`, StarReadingRenderer.java:172-209).
/// The client does not read the stack: no helper gives the window the main
/// hand stack outside the first-person hand pass.
#[derive(Debug, Clone, PartialEq)]
pub struct AmuletReading {
    /// Degrees clockwise from north, from the player to each place, in
    /// [0, 360): 0 is north (-Z), 90 east (+X). One per point, in the order of
    /// the points, then the world spawn last.
    pub azimuths: Vec<f32>,
    /// The name of each place, in the same order. The world spawn has an
    /// empty name. The spyglass shows a name that is not empty.
    pub names: Vec<String>,
    /// The index of the world spawn in both lists: its line is the lighter
    /// one. None for a reading with no spawn line (Java 1.7.0 and older, or a
    /// server that did not know the world spawn).
    pub spawn: Option<usize>,
    /// The game time of the reading (`Level.getGameTime`). A client that sees
    /// a reading more than 40 ticks old shows nothing (StarReadingRenderer.java:58).
    pub time: i64,
}

/// One of the six world rules of BMH (Java `BMHGameRules`,
/// BMHGameRules.java:27-37). On Horizon they are world settings of the mod,
/// not game rules (decision D5). The settings area keeps the values. Every
/// other area reads one through `bmh_settings::rule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Death sends the player on a distant respawn.
    RandomDeathSpawnPoint,
    /// The arrival shows the "new journey" message.
    DisplayTextOnRespawn,
    /// A night of sleep heals, costs food and gives two effects.
    FoodAndHealthRegenOnWakeUp,
    /// A player carries one Eternal Item of each type.
    OnlyCarryOneEternalItemType,
    /// A death wears the Eternal tools.
    DamageEternalItemWhenDead,
    /// Survival hides the coordinates of the debug screen.
    RemoveCoordinatesInSurvival,
}

impl Rule {
    /// The six rules, in the Java order (BMHGameRules.java:40, `ALL_RULES`).
    pub const ALL: [Rule; 6] = [
        Rule::RandomDeathSpawnPoint,
        Rule::DisplayTextOnRespawn,
        Rule::FoodAndHealthRegenOnWakeUp,
        Rule::OnlyCarryOneEternalItemType,
        Rule::DamageEternalItemWhenDead,
        Rule::RemoveCoordinatesInSurvival,
    ];

    /// The name of the rule: the path of the Java id `bmh:<name>`
    /// (BMHGameRules.java:27-37).
    pub fn name(self) -> &'static str {
        match self {
            Rule::RandomDeathSpawnPoint => "random_death_spawn_point",
            Rule::DisplayTextOnRespawn => "display_text_on_respawn",
            Rule::FoodAndHealthRegenOnWakeUp => "food_and_health_regen_on_wake_up",
            Rule::OnlyCarryOneEternalItemType => "only_carry_one_eternal_item_type",
            Rule::DamageEternalItemWhenDead => "damage_eternal_item_when_dead",
            Rule::RemoveCoordinatesInSurvival => "remove_coordinates_in_survival",
        }
    }

    /// The name of the rule in a world from before Minecraft 26.1
    /// (BMHGameRules.java:50-55).
    pub fn legacy_name(self) -> &'static str {
        match self {
            Rule::RandomDeathSpawnPoint => "randomDeathSpawnPoint",
            Rule::DisplayTextOnRespawn => "displayTextOnRespawn",
            Rule::FoodAndHealthRegenOnWakeUp => "foodAndHealthRegenOnWakeUp",
            Rule::OnlyCarryOneEternalItemType => "onlyCarryOneEternalItemType",
            Rule::DamageEternalItemWhenDead => "damageEternalItemWhenDead",
            Rule::RemoveCoordinatesInSurvival => "removeCoordinatesInSurvival",
        }
    }

    /// The rule with this name, in the new or the old spelling, with or
    /// without the `bmh:` namespace.
    pub fn from_name(name: &str) -> Option<Rule> {
        let path = name.strip_prefix("bmh:").unwrap_or(name);
        Rule::ALL
            .into_iter()
            .find(|rule| rule.name() == path || rule.legacy_name() == path)
    }

    /// The value in a new world. Every rule starts on
    /// (BMHGameRules.java:27-37, `registerBoolean(name, true)`).
    pub fn default_value(self) -> bool {
        true
    }
}

/// The values of the config file `config/bmh-common.toml` (Java
/// `NeoForgeConfig`, NeoForgeConfig.java:20-31). The settings area reads the
/// file. Every other area reads the values through `bmh_settings::config`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// `minDistanceFromBed`: the shortest distant respawn, in blocks.
    pub min_distance: i32,
    /// `maxDistanceFromBed`: the longest distant respawn, in blocks.
    pub max_distance: i32,
    /// `maxAmuletPoints`: the most points a player keeps on the amulet.
    pub max_amulet_points: i32,
}

impl Config {
    /// The lowest distance the file accepts (BMHConfigLimits.java:9).
    pub const MIN_DISTANCE: i32 = 5000;
    /// The highest distance the file accepts (BMHConfigLimits.java:10).
    pub const MAX_DISTANCE: i32 = 200_000;
    /// The fewest amulet points the file accepts (BMHConfigLimits.java:15).
    pub const MIN_AMULET_POINTS: i32 = 1;
    /// The most amulet points the file accepts (BMHConfigLimits.java:16).
    pub const MAX_AMULET_POINTS: i32 = 16;

    /// The values of a new config file (BMHConfigLimits.java:12-13, 17).
    pub const DEFAULT: Config = Config {
        min_distance: 5000,
        max_distance: 60_000,
        max_amulet_points: 5,
    };

    /// The same values, each one moved into its range, as `defineInRange`
    /// does when the file holds a value outside it.
    pub fn clamped(self) -> Config {
        Config {
            min_distance: self.min_distance.clamp(Self::MIN_DISTANCE, Self::MAX_DISTANCE),
            max_distance: self.max_distance.clamp(Self::MIN_DISTANCE, Self::MAX_DISTANCE),
            max_amulet_points: self
                .max_amulet_points
                .clamp(Self::MIN_AMULET_POINTS, Self::MAX_AMULET_POINTS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_names_are_the_java_ids() {
        assert_eq!(Rule::RandomDeathSpawnPoint.name(), "random_death_spawn_point");
        assert_eq!(Rule::RemoveCoordinatesInSurvival.name(), "remove_coordinates_in_survival");
        assert_eq!(Rule::ALL.len(), 6);
        for rule in Rule::ALL {
            assert!(rule.default_value(), "{}", rule.name());
        }
    }

    #[test]
    fn a_rule_is_found_by_each_spelling() {
        let rule = Some(Rule::DamageEternalItemWhenDead);
        assert_eq!(Rule::from_name("damage_eternal_item_when_dead"), rule);
        assert_eq!(Rule::from_name("bmh:damage_eternal_item_when_dead"), rule);
        assert_eq!(Rule::from_name("damageEternalItemWhenDead"), rule);
        assert_eq!(Rule::from_name("keep_inventory"), None);
    }

    #[test]
    fn the_config_defaults_are_the_java_defaults() {
        assert_eq!(Config::DEFAULT.min_distance, 5000);
        assert_eq!(Config::DEFAULT.max_distance, 60_000);
        assert_eq!(Config::DEFAULT.max_amulet_points, 5);
        assert_eq!(Config::DEFAULT.clamped(), Config::DEFAULT);
    }

    #[test]
    fn a_value_outside_its_range_moves_into_it() {
        let wild = Config { min_distance: 10, max_distance: 9_000_000, max_amulet_points: 0 };
        let tame = wild.clamped();
        assert_eq!(tame.min_distance, 5000);
        assert_eq!(tame.max_distance, 200_000);
        assert_eq!(tame.max_amulet_points, 1);
    }
}
