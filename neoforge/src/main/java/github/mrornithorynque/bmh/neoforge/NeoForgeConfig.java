package github.mrornithorynque.bmh.neoforge;

import net.neoforged.neoforge.common.ModConfigSpec;

/** config/bmh-common.toml: the same settings, defaults and range as the Forge build. */
public class NeoForgeConfig {

    public static final ModConfigSpec.Builder BUILDER = new ModConfigSpec.Builder();
    public static final ModConfigSpec SPEC;

    public static final ModConfigSpec.IntValue minDistanceFromBed;
    public static final ModConfigSpec.IntValue maxDistanceFromBed;
    public static final ModConfigSpec.IntValue maxAmuletPoints;

    private static final int MIN_DISTANCE_FROM_BED = 5000;
    private static final int MAX_DISTANCE_FROM_BED = 200000;

    static {
        minDistanceFromBed = BUILDER
            .comment("Minimum distance from bed to respawn")
            .defineInRange("minDistanceFromBed", 5000, MIN_DISTANCE_FROM_BED, MAX_DISTANCE_FROM_BED);

        maxDistanceFromBed = BUILDER
            .comment("Maximum distance from bed to respawn")
            .defineInRange("maxDistanceFromBed", 60000, MIN_DISTANCE_FROM_BED, MAX_DISTANCE_FROM_BED);

        maxAmuletPoints = BUILDER
            .comment("Most points a player can keep on the Navigator's Amulet")
            .defineInRange("maxAmuletPoints", 5, 1, 16);

        SPEC = BUILDER.build();
    }
}
