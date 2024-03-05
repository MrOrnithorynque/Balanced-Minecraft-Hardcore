package github.mrornithorynque.bmh;

import net.minecraftforge.common.ForgeConfigSpec;
import net.minecraftforge.fml.common.Mod;

@Mod.EventBusSubscriber(modid = BalancedMcHardcoreMain.MODID, bus = Mod.EventBusSubscriber.Bus.MOD)
public class Config {

    public static final ForgeConfigSpec.Builder BUILDER = new ForgeConfigSpec.Builder();
    public static final ForgeConfigSpec SPEC;

    public static final ForgeConfigSpec.ConfigValue<Integer> minDistanceFromBed;
    public static final ForgeConfigSpec.ConfigValue<Integer> maxDistanceFromBed;

    private static final int MIN_DISTANCE_FROM_BED = 5000;
    private static final int MAX_DISTANCE_FROM_BED = 200000;

    static {
        minDistanceFromBed = BUILDER
            .comment("Minimum distance from bed to respawn")
            .defineInRange("minDistanceFromBed", 5000, MIN_DISTANCE_FROM_BED, MAX_DISTANCE_FROM_BED);

        maxDistanceFromBed = BUILDER
            .comment("Maximum distance from bed to respawn")
            .defineInRange("maxDistanceFromBed", 60000, MIN_DISTANCE_FROM_BED, MAX_DISTANCE_FROM_BED);

        SPEC = BUILDER.build();
    }
}
