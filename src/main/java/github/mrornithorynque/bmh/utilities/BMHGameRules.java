package github.mrornithorynque.bmh.utilities;

import java.util.List;

import net.minecraft.world.level.GameRules;

public class BMHGameRules {

    public static final GameRules.Key<GameRules.BooleanValue> RULE_RANDOM_DEATH_SPAWN_POINT = GameRules
            .register("randomDeathSpawnPoint", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static final GameRules.Key<GameRules.BooleanValue> RULE_DISPLAY_TEXT_ON_RESPAWN = GameRules
            .register("displayTextOnRespawn", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static final GameRules.Key<GameRules.BooleanValue> RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP = GameRules
            .register("foodAndHealthRegenOnWakeUp", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static final GameRules.Key<GameRules.BooleanValue> RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE = GameRules
            .register("onlyCarryOneEternalItemType", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static final GameRules.Key<GameRules.BooleanValue> RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD = GameRules
            .register("damageEternalItemWhenDead", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static final GameRules.Key<GameRules.BooleanValue> RULE_REMOVE_COORDINATES_IN_SURVIVAL = GameRules
            .register("removeCoordinatesInSurvival", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

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
