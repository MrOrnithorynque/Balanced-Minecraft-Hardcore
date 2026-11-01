package github.mrornithorynque.bmh.utilities;

import java.util.List;

import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.world.level.GameRules;

public class BMHGameRules {

    public static final GameRules.Key<GameRules.BooleanValue> RULE_RANDOM_DEATH_SPAWN_POINT = Services.PLATFORM
            .registerBooleanGameRule("randomDeathSpawnPoint", GameRules.Category.MISC, true);

    public static final GameRules.Key<GameRules.BooleanValue> RULE_DISPLAY_TEXT_ON_RESPAWN = Services.PLATFORM
            .registerBooleanGameRule("displayTextOnRespawn", GameRules.Category.MISC, true);

    public static final GameRules.Key<GameRules.BooleanValue> RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP = Services.PLATFORM
            .registerBooleanGameRule("foodAndHealthRegenOnWakeUp", GameRules.Category.MISC, true);

    public static final GameRules.Key<GameRules.BooleanValue> RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE = Services.PLATFORM
            .registerBooleanGameRule("onlyCarryOneEternalItemType", GameRules.Category.MISC, true);

    public static final GameRules.Key<GameRules.BooleanValue> RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD = Services.PLATFORM
            .registerBooleanGameRule("damageEternalItemWhenDead", GameRules.Category.MISC, true);

    public static final GameRules.Key<GameRules.BooleanValue> RULE_REMOVE_COORDINATES_IN_SURVIVAL = Services.PLATFORM
            .registerBooleanGameRule("removeCoordinatesInSurvival", GameRules.Category.MISC, true);

    // Every BMH rule, flipped together by the world creation toggle.
    public static final List<GameRules.Key<GameRules.BooleanValue>> ALL_RULES = List.of(
            RULE_RANDOM_DEATH_SPAWN_POINT,
            RULE_DISPLAY_TEXT_ON_RESPAWN,
            RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP,
            RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE,
            RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD,
            RULE_REMOVE_COORDINATES_IN_SURVIVAL);

        public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
