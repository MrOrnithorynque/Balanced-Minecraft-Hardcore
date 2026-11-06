package github.mrornithorynque.bmh.utilities;

import java.util.List;

import com.mojang.brigadier.arguments.BoolArgumentType;
import com.mojang.serialization.Codec;

import github.mrornithorynque.bmh.platform.Services;

import net.minecraft.core.registries.Registries;
import net.minecraft.world.flag.FeatureFlagSet;
import net.minecraft.world.level.gamerules.GameRule;
import net.minecraft.world.level.gamerules.GameRuleCategory;
import net.minecraft.world.level.gamerules.GameRuleType;
import net.minecraft.world.level.gamerules.GameRuleTypeVisitor;

public class BMHGameRules {

    // Since 26.1 the game rules are a registry, with ids such as bmh:random_death_spawn_point.
    public static final GameRule<Boolean> RULE_RANDOM_DEATH_SPAWN_POINT = registerBoolean("random_death_spawn_point", true);

    public static final GameRule<Boolean> RULE_DISPLAY_TEXT_ON_RESPAWN = registerBoolean("display_text_on_respawn", true);

    public static final GameRule<Boolean> RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP = registerBoolean("food_and_health_regen_on_wake_up", true);

    public static final GameRule<Boolean> RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE = registerBoolean("only_carry_one_eternal_item_type", true);

    public static final GameRule<Boolean> RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD = registerBoolean("damage_eternal_item_when_dead", true);

    public static final GameRule<Boolean> RULE_REMOVE_COORDINATES_IN_SURVIVAL = registerBoolean("remove_coordinates_in_survival", true);

    // Every BMH rule, flipped together by the world creation toggle.
    public static final List<GameRule<Boolean>> ALL_RULES = List.of(
            RULE_RANDOM_DEATH_SPAWN_POINT,
            RULE_DISPLAY_TEXT_ON_RESPAWN,
            RULE_FOOD_AND_HEALTH_REGEN_ON_WAKE_UP,
            RULE_ONLY_CARRY_ONE_ETERNAL_ITEM_TYPE,
            RULE_DAMAGE_ETERNAL_ITEM_WHEN_DEAD,
            RULE_REMOVE_COORDINATES_IN_SURVIVAL);

    // The same rule object that vanilla's GameRules.registerBoolean makes. The loader registers it (Forge and NeoForge
    // do it later, Fabric at once), and the fields hold it from the start.
    private static GameRule<Boolean> registerBoolean(String name, boolean defaultValue) {

        GameRule<Boolean> rule = new GameRule<>(GameRuleCategory.MISC, GameRuleType.BOOL, BoolArgumentType.bool(),
                GameRuleTypeVisitor::visitBoolean, Codec.BOOL, value -> value ? 1 : 0, defaultValue, FeatureFlagSet.of());
        Services.PLATFORM.register(Registries.GAME_RULE, name, () -> rule);
        return rule;
    }

        public static void init() {
        /* This method is empty but ensures static fields are initialized. */ }
}
