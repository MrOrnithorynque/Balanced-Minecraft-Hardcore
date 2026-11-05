package github.mrornithorynque.bmh.neoforge;

import github.mrornithorynque.bmh.utilities.BMHConfigLimits;

import net.neoforged.neoforge.common.ModConfigSpec;

/** config/bmh-common.toml: the same settings, defaults and range as the Forge build. */
public class NeoForgeConfig {

    public static final ModConfigSpec.Builder BUILDER = new ModConfigSpec.Builder();
    public static final ModConfigSpec SPEC;

    public static final ModConfigSpec.IntValue minDistanceFromBed;
    public static final ModConfigSpec.IntValue maxDistanceFromBed;
    public static final ModConfigSpec.IntValue maxAmuletPoints;

    static {
        minDistanceFromBed = BUILDER
            .comment("Minimum distance from bed to respawn")
            .defineInRange("minDistanceFromBed", BMHConfigLimits.DEFAULT_MIN_DISTANCE_FROM_BED,
                BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);

        maxDistanceFromBed = BUILDER
            .comment("Maximum distance from bed to respawn")
            .defineInRange("maxDistanceFromBed", BMHConfigLimits.DEFAULT_MAX_DISTANCE_FROM_BED,
                BMHConfigLimits.MIN_DISTANCE_FROM_BED, BMHConfigLimits.MAX_DISTANCE_FROM_BED);

        maxAmuletPoints = BUILDER
            .comment("Most points a player can keep on the Navigator's Amulet")
            .defineInRange("maxAmuletPoints", BMHConfigLimits.DEFAULT_AMULET_POINTS,
                BMHConfigLimits.MIN_AMULET_POINTS, BMHConfigLimits.MAX_AMULET_POINTS);

        SPEC = BUILDER.build();
    }

    // The config screen calls it on the client thread. set() clears the cached value, save() writes the file.
    public static void save(int minDistance, int maxDistance, int amuletPoints) {

        minDistanceFromBed.set(minDistance);
        maxDistanceFromBed.set(maxDistance);
        maxAmuletPoints.set(amuletPoints);
        SPEC.save();
    }
}
