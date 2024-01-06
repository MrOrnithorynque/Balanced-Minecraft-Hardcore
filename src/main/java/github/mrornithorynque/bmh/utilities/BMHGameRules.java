package github.mrornithorynque.bmh.utilities;

import net.minecraft.world.level.GameRules;

public class BMHGameRules {

    public static final GameRules.Key<GameRules.BooleanValue> RULE_RANDOM_DEATH_SPAWN_POINT =
        GameRules.register("randomDeathSpawnPoint", GameRules.Category.MISC, GameRules.BooleanValue.create(true));

    public static void init() {
        // This method is empty but ensures static fields are initialized.
    }
}
