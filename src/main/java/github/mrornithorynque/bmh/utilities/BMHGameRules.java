package github.mrornithorynque.bmh.utilities;

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

    public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
